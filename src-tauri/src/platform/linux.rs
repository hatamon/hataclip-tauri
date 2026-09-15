use super::Point;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, InputFocus};

#[derive(Clone, Copy)]
pub struct Foreground {
    window: u32,
}

pub fn anchor_position() -> Option<Point> {
    pointer_position().or_else(focus_window_position)
}

pub fn capture_foreground() -> Option<Foreground> {
    let (conn, _screen_num) = x11rb::connect(None).ok()?;
    let reply = conn.get_input_focus().ok()?.reply().ok()?;
    if reply.focus < 2 {
        return None;
    }
    Some(Foreground {
        window: reply.focus,
    })
}

pub fn restore_foreground(fg: &Foreground) -> bool {
    let Ok((conn, _)) = x11rb::connect(None) else {
        return false;
    };
    conn.set_input_focus(InputFocus::PARENT, fg.window, 0u32)
        .is_ok()
        && conn.flush().is_ok()
}

fn pointer_position() -> Option<Point> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let root = conn.setup().roots.get(screen_num)?.root;
    let reply = conn.query_pointer(root).ok()?.reply().ok()?;
    Some(Point {
        x: i32::from(reply.root_x),
        y: i32::from(reply.root_y),
    })
}

fn focus_window_position() -> Option<Point> {
    let (conn, _screen_num) = x11rb::connect(None).ok()?;
    let focus = conn.get_input_focus().ok()?.reply().ok()?;
    if focus.focus < 2 {
        return None;
    }
    let geom = conn.get_geometry(focus.focus).ok()?.reply().ok()?;
    let root = geom.root;
    let translated = conn
        .translate_coordinates(focus.focus, root, 0, geom.height as i16)
        .ok()?
        .reply()
        .ok()?;
    Some(Point {
        x: i32::from(translated.dst_x),
        y: i32::from(translated.dst_y),
    })
}
