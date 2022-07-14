#include <X11/Xlib.h>
#include <iostream>

int main() {
  Display *display = XOpenDisplay(nullptr);
  if (display != nullptr) {
    std::cout << "Display is: " << display << "\n";
  } else {
    std::cerr << "diagonator: Unable to open the display. Make sure that "
                 "the $DISPLAY environment variable is set correctly.\n";
    return 1;
  }
}
