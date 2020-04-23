use super::controler::Builder;

// aliases for mod[1-5]
pub mod xmodmap {
    pub const MOD1: &str = "alt";
    pub const MOD2: &str = "Num_Lock";
    pub const MOD3: &str = "_mod3";
    pub const MOD4: &str = "super";
    pub const MOD5: &str = "_mod5";
}

pub fn bind(b: &mut Builder) {
    bind_custom(b);
    bind_bspc(b);
}

fn bind_custom(b: &mut Builder) {
    b.bind("super + a", "/home/hyyking/.xmonad/scripts/browser.sh");
    b.bind("super + e", "alacritty");
    b.bind("super + z", "pcmanfm");
    b.bind("super + q", "dmenu_run");
}

fn bind_bspc(b: &mut Builder) {
    // quit/restart bspwm
    b.bind("super + alt + q", "bspc quit");
    b.bind("super + alt + r", "bspc wm -r");

    // close and kill
    b.bind("super + c", "bspc node -c");
    b.bind("super + shift + c", "bspc node -k");

    // alternate between the tiled and monocle layout
    b.bind("super + m", "bspc desktop -l next");

    // send the newest marked node to the newest preselected node
    b.bind(
        "super + y",
        "bspc node newest.marker.local -n newest.!automatic.local",
    );

    // swap the current node and the biggest node
    b.bind("super + g", "bspc node -s biggest");

    // set the window state
    b.bind("super + t", "bspc node -t tiled");
    b.bind("super + shift + t", "bspc node -t pseudo_tiled");
    b.bind("super + s", "bspc node -t floating");
    b.bind("super + f", "bspc node -t fullscreen");

    // set the node flags
    b.bind("super + ctrl + m", "bspc node -g marked");
    b.bind("super + ctrl + x", "bspc node -g locked");
    b.bind("super + ctrl + y", "bspc node -g sticky");
    b.bind("super + ctrl + z", "bspc node -g private");

    // focus the node in the given direction
    b.bind("super + h", "bspc node -f west");
    b.bind("super + j", "bspc node -f south");
    b.bind("super + k", "bspc node -f north");
    b.bind("super + l", "bspc node -f east");
    b.bind("super + shift + h", "bspc node -s west");
    b.bind("super + shift + j", "bspc node -s south");
    b.bind("super + shift + k", "bspc node -s north");
    b.bind("super + shift + l", "bspc node -s east");

    // focus the node for the given path jump
    b.bind("super + p", "bspc node -f @parent");
    b.bind("super + b", "bspc node -f @brother");
    b.bind("super + comma", "bspc node -f @first");
    b.bind("super + period", "bspc node -f @second");

    // focus the next/previous node in the current desktop
    b.bind("super + u", "bspc node -f next.local");
    b.bind("super + shift + u", "bspc node -f prev.local");

    // focus the next/previous desktop in the current monitor
    b.bind("super + bracketleft", "bspc desktop -f prev.local");
    b.bind("super + bracketright", "bspc desktop -f next.local");

    //  focus the last node/desktop
    b.bind("super + grave", "bspc node -f last");
    b.bind("super + Tab", "bspc desktop -f last");

    // focus or send to the given desktop
    b.bind("ctrl + alt + Left", "bspc desktop -f prev.local");
    b.bind("ctrl + alt + Right", "bspc desktop -f next.local");
    b.bind(
        "ctrl + alt + shift + Left",
        "bspc node -d prev.local --follow",
    );
    b.bind(
        "ctrl + alt + shift + Right",
        "bspc node -d next.local --follow",
    );

    // preselect the direction
    b.bind("super + ctrl + h", "bspc node -p west");
    b.bind("super + ctrl + j", "bspc node -p south");
    b.bind("super + ctrl + k", "bspc node -p north");
    b.bind("super + ctrl + l", "bspc node -p east");

    // preselect the ratio
    b.bind("super + ctrl + 1", "bspc node -o 0.1");
    b.bind("super + ctrl + 2", "bspc node -o 0.2");
    b.bind("super + ctrl + 3", "bspc node -o 0.3");
    b.bind("super + ctrl + 4", "bspc node -o 0.4");
    b.bind("super + ctrl + 5", "bspc node -o 0.5");
    b.bind("super + ctrl + 6", "bspc node -o 0.6");
    b.bind("super + ctrl + 7", "bspc node -o 0.7");
    b.bind("super + ctrl + 8", "bspc node -o 0.8");
    b.bind("super + ctrl + 9", "bspc node -o 0.9");

    // cancel the preselection for the focused node
    b.bind("super + ctrl + space", "bspc node -p cancel");

    // expand a window by moving one of its side outward
    b.bind("super + alt + h", "bspc node -z left -20 0");
    b.bind("super + alt + j", "bspc node -z bottom 0 20");
    b.bind("super + alt + k", "bspc node -z top 0 -20");
    b.bind("super + alt + l", "bspc node -z right 20 0");

    // contract a window by moving one of its side inward
    b.bind("super + alt + shift + h", "bspc node -z right -20 0");
    b.bind("super + alt + shift + j", "bspc node -z top 0 20");
    b.bind("super + alt + shift + k", "bspc node -z bottom 0 -20");
    b.bind("super + alt + shift + l", "bspc node -z left 20 0");

    // move floating window
    b.bind("super + Left", "bspc node -v -20 0");
    b.bind("super + Down", "bspc node -v 0 20");
    b.bind("super + Up", "bspc node -v 0 -20");
    b.bind("super + Right", "bspc node -v 20 0");

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
