#include <X11/Xlib.h>
#include <iostream>

int main() {
  Display *display = XOpenDisplay(nullptr);
  if (display == nullptr) {
    std::cerr << "diagonator: Unable to open display '" << XDisplayName(nullptr)
              << "'. Make sure that the $DISPLAY environment variable is set "
                 "correctly.\n";
    return 1;
  }
  std::cout << "Display is: " << display << "\n";
  XCloseDisplay(display);
}
