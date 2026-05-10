use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt;

pub fn register_wm<C: Connection>(conn: &C, root: Window) {
    let net_wm_name =
        x11rb::protocol::xproto::ConnectionExt::intern_atom(&conn, false, b"_NET_WM_NAME")
            .unwrap()
            .reply()
            .unwrap()
            .atom;

    let utf8 = x11rb::protocol::xproto::ConnectionExt::intern_atom(&conn, false, b"UTF8_STRING")
        .unwrap()
        .reply()
        .unwrap()
        .atom;

    x11rb::protocol::xproto::ConnectionExt::change_property8(
        &conn,
        PropMode::REPLACE,
        root,
        net_wm_name,
        utf8,
        b"OrbitWM",
    )
    .unwrap();

    let wm_check = x11rb::protocol::xproto::ConnectionExt::generate_id(&conn).unwrap();

    x11rb::protocol::xproto::ConnectionExt::create_window(
        &conn,
        COPY_DEPTH_FROM_PARENT,
        wm_check,
        root,
        0,
        0,
        1,
        1,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::default(),
    )
    .unwrap();

    let net_supporting = x11rb::protocol::xproto::ConnectionExt::intern_atom(
        &conn,
        false,
        b"_NET_SUPPORTING_WM_CHECK",
    )
    .unwrap()
    .reply()
    .unwrap()
    .atom;

    x11rb::protocol::xproto::ConnectionExt::change_property32(
        &conn,
        PropMode::REPLACE,
        root,
        net_supporting,
        AtomEnum::WINDOW,
        &[wm_check],
    )
    .unwrap();

    let _ = conn.flush();
}

pub fn get_window_type<C: Connection>(conn: &C, window: Window) -> Option<Atom> {
    let atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
        .ok()?
        .reply()
        .ok()?
        .atom;

    let reply = conn
        .get_property(false, window, atom, AtomEnum::ATOM, 0, 1)
        .ok()?
        .reply()
        .ok()?;

    reply.value32().and_then(|mut v| v.next())
}
