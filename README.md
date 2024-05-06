Go to `http://[::1]:8080/` when running `trunk serve` from WSL2.

https://github.com/emilk/eframe_template/

https://github.com/emilk/eframe_template/blob/main/src/main.rs

# Notes

How to make things greyed out:
```
if ui.add(egui::Button::new("Click me")).clicked() {
    do_stuff();
}

// A greyed-out and non-interactive button:
if ui.add_enabled(false, egui::Button::new("Can't click this")).clicked() {
    unreachable!();
}
```

To toggle dark mode in a ui
```
ui.style_mut().visuals = Visuals::dark();
```
or globally
```
ctx.set_visuals(Visuals::dark());
```

# Workarounds
If, when using `trunk serve` on Windows:
```
ERROR error from server task error=An attempt was made to access a socket in a way forbidden by its access permissions. (os error 10013)
ERROR An attempt was made to access a socket in a way forbidden by its access permissions. (os error 10013)
```
then
```
netstat -ano | find ":80"
TASKKILL /F /PID 0000
```