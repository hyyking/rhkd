use super::controler::Builder;

pub fn bind(b: &mut Builder) {
    bind_custom(b);
    bind_bspc(b);
}

fn bind_custom(b: &mut Builder) {
    b.bind("mod4 + a", "/home/hyyking/.xmonad/scripts/browser.sh");
    b.bind("mod4 + e", "alacritty");
    b.bind("mod4 + z", "pcmanfm");
    b.bind("mod4 + q", "dmenu_run");
}

fn bind_bspc(b: &mut Builder) {
    // quit/restart bspwm
    b.bind("mod4 + mod1 + q", "bspc quit");
    b.bind("mod4 + mod1 + r", "bspc wm -r");

    // close and kill
    b.bind("mod4 + c", "bspc node -c");
    b.bind("mod4 + shift + c", "bspc node -k");

    // alternate between the tiled and monocle layout
    b.bind("mod4 + m", "bspc desktop -l next");

    // send the newest marked node to the newest preselected node
    b.bind(
        "mod4 + y",
        "bspc node newest.marker.local -n newest.!automatic.local",
    );

    // swap the current node and the biggest node
    b.bind("mod4 + g", "bspc node -s biggest");

    // set the window state
    b.bind("mod4 + t", "bspc node -t tiled");
    b.bind("mod4 + shift + t", "bspc node -t pseudo_tiled");
    b.bind("mod4 + s", "bspc node -t floating");
    b.bind("mod4 + f", "bspc node -t fullscreen");

    // set the node flags
    b.bind("mod4 + ctrl + m", "bspc node -g marked");
    b.bind("mod4 + ctrl + x", "bspc node -g locked");
    b.bind("mod4 + ctrl + y", "bspc node -g sticky");
    b.bind("mod4 + ctrl + z", "bspc node -g private");

    // focus the node in the given direction
    b.bind("mod4 + h", "bspc node -f west");
    b.bind("mod4 + j", "bspc node -f south");
    b.bind("mod4 + k", "bspc node -f north");
    b.bind("mod4 + l", "bspc node -f east");
    b.bind("mod4 + shift + h", "bspc -s west");
    b.bind("mod4 + shift + j", "bspc -s south");
    b.bind("mod4 + shift + k", "bspc -s north");
    b.bind("mod4 + shift + l", "bspc -s east");

    // focus the node for the given path jump
    b.bind("mod4 + p", "bspc node -f @parent");
    b.bind("mod4 + b", "bspc node -f @brother");
    b.bind("mod4 + comma", "bspc node -f @first");
    b.bind("mod4 + period", "bspc node -f @second");

    // focus the next/previous node in the current desktop
    b.bind("mod4 + u", "bspc node -f next.local");
    b.bind("mod4 + shift + u", "bspc node -f prev.local");

    // focus the next/previous desktop in the current monitor
    b.bind("mod4 + bracketleft", "bspc desktop -f prev.local");
    b.bind("mod4 + bracketright", "bspc desktop -f next.local");

    //  focus the last node/desktop
    b.bind("mod4 + grave", "bspc node -f last");
    b.bind("mod4 + Tab", "bspc desktop -f last");

    // focus or send to the given desktop
    b.bind("ctrl + mod1 + Left", "bspc desktop -f prev.local");
    b.bind("ctrl + mod1 + Right", "bspc desktop -f next.local");
    b.bind(
        "ctrl + mod1 + shift + Left",
        "bspc node -d prev.local --follow",
    );
    b.bind(
        "ctrl + mod1 + shift + Right",
        "bspc node -d next.local --follow",
    );

    // preselect the direction
    b.bind("mod4 + ctrl + h", "bspc node -p west");
    b.bind("mod4 + ctrl + j", "bspc node -p south");
    b.bind("mod4 + ctrl + k", "bspc node -p north");
    b.bind("mod4 + ctrl + l", "bspc node -p east");

    // preselect the ratio
    b.bind("mod4 + ctrl + 1", "bspc node -o 0.1");
    b.bind("mod4 + ctrl + 2", "bspc node -o 0.2");
    b.bind("mod4 + ctrl + 3", "bspc node -o 0.3");
    b.bind("mod4 + ctrl + 4", "bspc node -o 0.4");
    b.bind("mod4 + ctrl + 5", "bspc node -o 0.5");
    b.bind("mod4 + ctrl + 6", "bspc node -o 0.6");
    b.bind("mod4 + ctrl + 7", "bspc node -o 0.7");
    b.bind("mod4 + ctrl + 8", "bspc node -o 0.8");
    b.bind("mod4 + ctrl + 9", "bspc node -o 0.9");

    // cancel the preselection for the focused node
    b.bind("mod4 + ctrl + space", "bspc node -p cancel");

    // expand a window by moving one of its side outward
    b.bind("mod4 + mod1 + h", "bspc node -z left -20 0");
    b.bind("mod4 + mod1 + j", "bspc node -z bottom 0 20");
    b.bind("mod4 + mod1 + k", "bspc node -z top 0 -20");
    b.bind("mod4 + mod1 + l", "bspc node -z right 20 0");

    // contract a window by moving one of its side inward
    b.bind("mod4 + mod1 + shift + h", "bspc node -z right -20 0");
    b.bind("mod4 + mod1 + shift + j", "bspc node -z top 0 20");
    b.bind("mod4 + mod1 + shift + k", "bspc node -z bottom 0 -20");
    b.bind("mod4 + mod1 + shift + l", "bspc node -z left 20 0");

    // move floating window
    b.bind("mod4 + Left", "bspc node -v -20 0");
    b.bind("mod4 + Down", "bspc node -v 0 20");
    b.bind("mod4 + Up", "bspc node -v 0 -20");
    b.bind("mod4 + Right", "bspc node -v 20 0");

    // ################################### UNBOUNDED ###################################
    // # focus the older or newer node in the focus history
    // super + {o,i}
    //         bspc wm -h off; \
    //         bspc node {older,newer} -f; \
    //         bspc wm -h on
    //
    // # cancel the preselection for the focused desktop
    // super + ctrl + shift + space
    //         bspc query -N -d | xargs -I id -n 1 bspc node id -p cancel
}
