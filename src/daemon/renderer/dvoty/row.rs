use std::{cell::RefMut, time::Duration};

use glib::object::CastNone;
use gtk4::{
    prelude::{AdjustmentExt, WidgetExt},
    ListBoxRow, ScrolledWindow, Viewport, Window,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::app::AppContext,
        structs::{DaemonCmd, DaemonEvt, Dvoty},
    },
    utils::DaemonErr,
};

use super::DvotyTaskType;

async fn murph(sender: UnboundedSender<DaemonEvt>, target: f64, mut current: f64) {
    for _ in 0..100 {
        current += (target - current) * 0.15f64;
        sender
            .send(DaemonEvt {
                evt: DaemonCmd::Dvoty(Dvoty::SetScroll(current)),
                sender: None,
            })
            .unwrap_or_else(|e| {
                println!("Dvoty: Can't send scroll: {}", e);
            });
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::SetScroll(target)),
            sender: None,
        })
        .unwrap_or_else(|e| {
            println!("Dvoty: Can't send scroll: {}", e);
        });
}

fn init_murph(
    context_ref: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
    current: f64,
) {
    if context_ref
        .dvoty
        .dvoty_tasks
        .contains_key(&DvotyTaskType::MurphViewport)
    {
        context_ref
            .dvoty
            .dvoty_tasks
            .get(&DvotyTaskType::MurphViewport)
            .unwrap()
            .abort();
        context_ref
            .dvoty
            .dvoty_tasks
            .remove(&DvotyTaskType::MurphViewport);
    }

    let target = context_ref.dvoty.target_scroll;

    let handle = tokio::spawn(async move {
        murph(sender, target, current).await;
    });

    context_ref
        .dvoty
        .dvoty_tasks
        .insert(DvotyTaskType::MurphViewport, handle);
}

fn adjust_row(
    row: ListBoxRow,
    viewport: Viewport,
    scroll: ScrolledWindow,
    context_ref: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
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
        context_ref.dvoty.target_scroll =
            adjustment.value() - (viewport_bound.y() - row_bound.y()) as f64;
        init_murph(context_ref, sender, adjustment.value());
    } else if row_bound.y() + row_bound.height() > viewport_bound.y() + viewport_bound.height() {
        // if the bottom of the row is not in the viewport, increase the adjustment value by the
        // difference
        context_ref.dvoty.target_scroll = adjustment.value()
            + (row_bound.y() + row_bound.height() - viewport_bound.y() - viewport_bound.height())
                as f64;
        init_murph(context_ref, sender, adjustment.value());
    }

    Ok(())
}

pub fn set_scroll(
    context_ref: &mut RefMut<AppContext>,
    window: &Window,
    value: f64,
) -> Result<(), DaemonErr> {
    let scroll = if let Some(s) = &context_ref.dvoty.dvoty_scroll {
        s
    } else {
        if let Ok(res) = super::utils::get_scrolled_window(window) {
            context_ref.dvoty.dvoty_scroll = Some(res);
            &context_ref.dvoty.dvoty_scroll.as_ref().unwrap()
        } else {
            println!("Dvoty: can't find scrolled window");
            return Err(DaemonErr::CannotFindWidget);
        }
    };

    scroll.vadjustment().set_value(value);

    Ok(())
}

pub fn ensure_row_in_viewport(
    context_ref: &mut RefMut<AppContext>,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
) -> Result<(), DaemonErr> {
    let scroll = if let Some(s) = &context_ref.dvoty.dvoty_scroll {
        s
    } else {
        if let Ok(res) = super::utils::get_scrolled_window(window) {
            context_ref.dvoty.dvoty_scroll = Some(res);
            &context_ref.dvoty.dvoty_scroll.as_ref().unwrap()
        } else {
            println!("Dvoty: can't find scrolled window");
            return Err(DaemonErr::CannotFindWidget);
        }
    };

    let viewport = if let Some(v) = scroll.first_child().and_downcast::<gtk4::Viewport>() {
        v
    } else {
        println!("Dvoty: can't find viewport");

        return Err(DaemonErr::CannotFindWidget);
    };

    let row = context_ref.dvoty.dvoty_entries[context_ref.dvoty.cur_ind]
        .1
        .clone();

    adjust_row(row, viewport, scroll.clone(), context_ref, sender)?;

    Ok(())
}
