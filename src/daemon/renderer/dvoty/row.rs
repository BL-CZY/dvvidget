use std::time::Duration;

use glib::object::CastNone;
use gtk4::{
    prelude::{AdjustmentExt, WidgetExt},
    ListBoxRow, ScrolledWindow, Viewport, Window,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::structs::{DaemonCmdType, DaemonEvt, Dvoty},
    utils::DaemonErr,
};

use super::{DvotyContext, DvotyTaskType};

async fn murph(sender: UnboundedSender<DaemonEvt>, target: f64, mut current: f64, monitor: usize) {
    for _ in 0..100 {
        current += (target - current) * 0.15f64;
        sender
            .send(DaemonEvt {
                evt: DaemonCmdType::Dvoty(Dvoty::SetScroll(current)),
                sender: None,
                uuid: None,
                monitor,
            })
            .unwrap_or_else(|e| {
                println!("Dvoty: Can't send scroll: {}", e);
            });
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::SetScroll(target)),
            sender: None,
            uuid: None,
            monitor,
        })
        .unwrap_or_else(|e| {
            println!("Dvoty: Can't send scroll: {}", e);
        });
}

fn init_murph(
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    current: f64,
    monitor: usize,
) {
    if context.dvoty_tasks[monitor].contains_key(&DvotyTaskType::MurphViewport) {
        context.dvoty_tasks[monitor]
            .get(&DvotyTaskType::MurphViewport)
            .unwrap()
            .abort();
        context.dvoty_tasks[monitor].remove(&DvotyTaskType::MurphViewport);
    }

    let target = context.target_scroll[monitor];

    let handle = tokio::spawn(async move {
        murph(sender, target, current, monitor).await;
    });

    context.dvoty_tasks[monitor].insert(DvotyTaskType::MurphViewport, handle);
}

fn adjust_row(
    row: ListBoxRow,
    viewport: Viewport,
    scroll: ScrolledWindow,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) -> Result<(), DaemonErr> {
    let viewport_bound = if let Some(bound) = viewport.compute_bounds(&viewport) {
        bound
    } else {
        return Err(DaemonErr::CannotFindWidget);
    };

    let row_bound = if let Some(bound) = row.compute_bounds(&viewport) {
        bound
    } else {
        return Err(DaemonErr::CannotFindWidget);
    };

    let adjustment = scroll.vadjustment();

    if row_bound.y() < viewport_bound.y() {
        // if the top of the row is not in the viewport, reduce the adjustment value by the
        // difference
        context.target_scroll[monitor] =
            adjustment.value() - (viewport_bound.y() - row_bound.y()) as f64;
        init_murph(context, sender, adjustment.value(), monitor);
    } else if row_bound.y() + row_bound.height() > viewport_bound.y() + viewport_bound.height() {
        // if the bottom of the row is not in the viewport, increase the adjustment value by the
        // difference
        context.target_scroll[monitor] = adjustment.value()
            + (row_bound.y() + row_bound.height() - viewport_bound.y() - viewport_bound.height())
                as f64;
        init_murph(context, sender, adjustment.value(), monitor);
    }

    Ok(())
}

pub fn set_scroll(
    context: &mut DvotyContext,
    window: &Window,
    value: f64,
    monitor: usize,
) -> Result<(), DaemonErr> {
    let scroll = if let Some(s) = &context.dvoty_scroll[monitor] {
        s
    } else if let Ok(res) = super::utils::get_scrolled_window(window) {
        context.dvoty_scroll[monitor] = Some(res);
        context.dvoty_scroll[monitor].as_ref().unwrap()
    } else {
        println!("Dvoty: can't find scrolled window");
        return Err(DaemonErr::CannotFindWidget);
    };

    scroll.vadjustment().set_value(value);

    Ok(())
}

pub fn ensure_row_in_viewport(
    context: &mut DvotyContext,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) -> Result<(), DaemonErr> {
    let scroll = if let Some(s) = &context.dvoty_scroll[monitor] {
        s
    } else if let Ok(res) = super::utils::get_scrolled_window(window) {
        context.dvoty_scroll[monitor] = Some(res);
        context.dvoty_scroll[monitor].as_ref().unwrap()
    } else {
        println!("Dvoty: can't find scrolled window");
        return Err(DaemonErr::CannotFindWidget);
    };

    let viewport = if let Some(v) = scroll.first_child().and_downcast::<gtk4::Viewport>() {
        v
    } else {
        println!("Dvoty: can't find viewport");

        return Err(DaemonErr::CannotFindWidget);
    };

    let row = context.dvoty_entries[monitor][context.cur_ind[monitor]]
        .1
        .clone();

    adjust_row(row, viewport, scroll.clone(), context, sender, monitor)?;

    Ok(())
}
