use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use chrono::Local;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus_crossroads::{Context, Crossroads};

use crate::utils::shutdown;

const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";

// Type aliases for the notification actions and hints
type Actions = Vec<String>;
type Hints = HashMap<String, dbus::arg::Variant<Box<dyn dbus::arg::RefArg + 'static>>>;

// Struct to hold notification data
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub timestamp: SystemTime,
}

// Struct to hold server state
struct NotificationServer {
    notifications: HashMap<u32, Notification>,
    next_id: u32,
}

impl NotificationServer {
    fn new() -> Self {
        NotificationServer {
            notifications: HashMap::new(),
            next_id: 1,
        }
    }

    // Register interface methods on the given crossroads instance
    fn register_interface(
        cr: &mut Crossroads,
    ) -> dbus_crossroads::IfaceToken<Arc<Mutex<NotificationServer>>> {
        cr.register(NOTIFICATIONS_INTERFACE, |b| {
            // Notify method
            b.method(
                "Notify",
                (
                    "app_name",
                    "replaces_id",
                    "app_icon",
                    "summary",
                    "body",
                    "actions",
                    "hints",
                    "expire_timeout",
                ),
                ("id",),
                |ctx: &mut Context,
                 server: &mut Arc<Mutex<NotificationServer>>,
                 (
                    app_name,
                    replaces_id,
                    app_icon,
                    summary,
                    body,
                    actions,
                    hints,
                    expire_timeout,
                ): (String, u32, String, String, String, Actions, Hints, i32)| {
                    let notification_id = handle_notify(
                        server,
                        app_name,
                        replaces_id,
                        app_icon,
                        summary,
                        body,
                        actions,
                        hints,
                        expire_timeout,
                    );
                    Ok((notification_id,))
                },
            );

            // GetCapabilities method
            b.method(
                "GetCapabilities",
                (),
                ("capabilities",),
                |_: &mut Context, _: &mut Arc<Mutex<NotificationServer>>, _: ()| {
                    let capabilities = vec!["body".to_string(), "body-markup".to_string()];
                    Ok((capabilities,))
                },
            );

            // CloseNotification method
            b.method(
                "CloseNotification",
                ("id",),
                (),
                |ctx: &mut Context, server: &mut Arc<Mutex<NotificationServer>>, (id,): (u32,)| {
                    handle_close_notification(ctx, server, id);
                    Ok(())
                },
            );

            // GetServerInformation method
            b.method(
                "GetServerInformation",
                (),
                ("name", "vendor", "version", "spec_version"),
                |_: &mut Context, _: &mut Arc<Mutex<NotificationServer>>, _: ()| {
                    Ok((
                        "Denote".to_string(),
                        "Bl-CZY".to_string(),
                        "1.0".to_string(),
                        "1.2".to_string(),
                    ))
                },
            );

            // NotificationClosed signal
            b.signal::<(u32, u32), _>("NotificationClosed", ("id", "reason"));
        })
    }
}

// Handle the Notify method call
fn handle_notify(
    server: &Arc<Mutex<NotificationServer>>,
    app_name: String,
    replaces_id: u32,
    _app_icon: String,
    summary: String,
    body: String,
    _actions: Actions,
    _hints: Hints,
    _expire_timeout: i32,
) -> u32 {
    let mut server_lock = server.lock().unwrap();
    let notification_id: u32;

    // Check if this is replacing an existing notification
    if replaces_id > 0 && server_lock.notifications.contains_key(&replaces_id) {
        notification_id = replaces_id;
    } else {
        notification_id = server_lock.next_id;
        server_lock.next_id += 1;
    }

    // Create and store the notification
    let notification = Notification {
        id: notification_id,
        app_name: app_name.clone(),
        summary: summary.clone(),
        body: body.clone(),
        timestamp: SystemTime::now(),
    };

    server_lock
        .notifications
        .insert(notification_id, notification.clone());

    // Print notification details
    let time = Local::now().format("%H:%M:%S").to_string();
    println!(
        "[{}] Notification #{} from {}: {}",
        time, notification_id, app_name, summary
    );
    if !body.is_empty() {
        println!("  Body: {}", body);
    }

    notification_id
}

// Handle the CloseNotification method call
fn handle_close_notification(ctx: &mut Context, server: &Arc<Mutex<NotificationServer>>, id: u32) {
    let mut server_lock = server.lock().unwrap();

    if server_lock.notifications.remove(&id).is_some() {
        println!("Closing notification #{}", id);

        // Emit NotificationClosed signal (reason 3 = closed by CloseNotification call)
        let reason: u32 = 3;
        let signal = dbus::message::Message::signal(
            &NOTIFICATIONS_PATH.into(),
            &NOTIFICATIONS_INTERFACE.into(),
            &"NotificationClosed".into(),
        )
        .append2(id, reason);

        ctx.push_msg(signal);
    }
}

pub async fn start_notification_server(
) -> Result<(tokio::task::JoinHandle<()>, Arc<SyncConnection>), Box<dyn std::error::Error>> {
    // Create a new Crossroads instance
    let mut cr = Crossroads::new();

    // Connect to the session bus
    let (resource, connection) = dbus_tokio::connection::new_session_sync()?;

    cr.set_async_support(Some((
        connection.clone(),
        Box::new(|x| {
            tokio::spawn(x);
        }),
    )));

    // Create server state and register interface
    let server = Arc::new(Mutex::new(NotificationServer::new()));
    let iface_token = NotificationServer::register_interface(&mut cr);

    // Register server state at object path
    cr.insert(NOTIFICATIONS_PATH, &[iface_token], server);

    connection.start_receive(
        MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn)
                .unwrap_or_else(|_| println!("Denote: Failed to handle message"));
            true
        }),
    );

    let handle = tokio::spawn(async {
        let err = resource.await;
        shutdown(&format!(
            "Denote: lost connection from the dbus server: {}",
            err
        ));
    });

    // Request the Notifications service name
    connection
        .request_name(NOTIFICATIONS_INTERFACE, false, true, false)
        .await?;

    Ok((handle, connection))
}
