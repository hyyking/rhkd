use std::{ffi::OsStr, process::Command};

use super::hotkeys::Builder;

use rhkb_lib::keyboard::{french::*, *};

pub fn bind(ctrl: &mut Builder) {
    bind_user(ctrl);
    bind_bspwm(ctrl);
}

fn bind_user(ctrl: &mut Builder) {
    ctrl.bind(&[MOD, E], Command::new("alacritty"));
    ctrl.bind(&[MOD, Z], Command::new("pcmanfm"));
    ctrl.bind(&[MOD, A], shell("/home/hyyking/.xmonad/scripts/browser.sh"));
    ctrl.bind(&[MOD, Q], Command::new("dmenu_run"));
}

fn bind_bspwm(ctrl: &mut Builder) {
    // switch workspaces
    ctrl.bind(&[CTRL, ALT, RIGHT], bspc(&["desktop", "-f", "next.local"]));
    ctrl.bind(&[CTRL, ALT, LEFT], bspc(&["desktop", "-f", "prev.local"]));

    // move window to different workspace
    ctrl.bind(
        &[CTRL, L_SHIFT, ALT, LEFT],
        bspc(&["node", "-d", "prev.local", "--follow"]),
    );
    ctrl.bind(
        &[CTRL, L_SHIFT, ALT, RIGHT],
        bspc(&["node", "-d", "next.local", "--follow"]),
    );

    // reload bspwm
    ctrl.bind(&[MOD, ALT, R], bspc(&["wm", "-r"]));

    // super + {_,shift + }c
    //     bspc node -{c,k}
    ctrl.bind(&[MOD, C], bspc(&["node", "-c"]));
    ctrl.bind(&[MOD, L_SHIFT, C], bspc(&["node", "-k"]));
}

/*

# quit/restart bspwm
super + alt + {q,r}
    bspc {quit,wm -r}

# close and kill
super + {_,shift + }c
    bspc node -{c,k}
 * */

fn bspc<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("bspc");
    cmd.args(args);
    cmd
}

fn shell<S: AsRef<OsStr>>(script: S) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg(script);
    cmd
}
