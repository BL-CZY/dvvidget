use std::cell::RefMut;

use glib::object::CastNone;
use gtk4::{
    prelude::{AdjustmentExt, WidgetExt},
    ListBoxRow, ScrolledWindow, Viewport, Window,
};

use crate::{daemon::renderer::app::AppContext, utils::DaemonErr};

fn adjust_row(
    row: ListBoxRow,
    viewport: Viewport,
    scroll: ScrolledWindow,
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
        adjustment.set_value(adjustment.value() - (viewport_bound.y() - row_bound.y()) as f64);
    } else if row_bound.y() + row_bound.height() > viewport_bound.y() + viewport_bound.height() {
        // if the bottom of the row is not in the viewport, increase the adjustment value by the
        // difference
        adjustment.set_value(
            adjustment.value()
                + (row_bound.y() + row_bound.height()
                    - viewport_bound.y()
                    - viewport_bound.height()) as f64,
        );
    }

    Ok(())
}

pub fn ensure_row_in_viewport(
    context_ref: &mut RefMut<AppContext>,
    window: &Window,
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

    adjust_row(row, viewport, scroll.clone())?;

    Ok(())
}
