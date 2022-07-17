#include <X11/Xlib.h>
#include <X11/extensions/Xcomposite.h>
#include <iostream>

int main() {
  Display *display = XOpenDisplay(nullptr);
  if (display == nullptr) {
    std::cerr << "diagonator: Unable to open display '" << XDisplayName(nullptr)
              << "'. Make sure that the $DISPLAY environment variable is set "
                 "correctly.\n";
    return 1;
  }
  int default_screen = XDefaultScreen(display);
  int root_window = XRootWindow(display, default_screen);
  std::cout << "Root is: " << root_window << '\n';
  std::cout << "Window is: " << XCompositeGetOverlayWindow(display, root_window)
            << "\n";
  XCloseDisplay(display);
}
