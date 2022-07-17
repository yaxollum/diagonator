#include <X11/Xlib.h>
#include <X11/extensions/Xcomposite.h>
#include <X11/extensions/Xfixes.h>
#include <X11/extensions/shape.h>
#include <chrono>
#include <iostream>
#include <thread>

int main() {
  Display *display = XOpenDisplay(nullptr);
  if (display == nullptr) {
    std::cerr << "diagonator: Unable to open display '" << XDisplayName(nullptr)
              << "'. Make sure that the $DISPLAY environment variable is set "
                 "correctly.\n";
    return 1;
  }
  Window root_window = XDefaultRootWindow(display);
  Window overlay_window = XCompositeGetOverlayWindow(display, root_window);

  // Allow input passthrough for overlay.
  // From https://github.com/MrBober/x11-overlay/blob/master/overlay.c
  XserverRegion region = XFixesCreateRegion(display, nullptr, 0);
  XFixesSetWindowShapeRegion(display, overlay_window, ShapeBounding, 0, 0, 0);
  XFixesSetWindowShapeRegion(display, overlay_window, ShapeInput, 0, 0, region);
  XFixesDestroyRegion(display, region);

  XFlush(display);
  XEvent ev;
  while (true) {
    XNextEvent(display, &ev);
  }
  XCloseDisplay(display);
}
