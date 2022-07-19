/*
 * Copyright © 2003 Keith Packard
 *
 * Permission to use, copy, modify, distribute, and sell this software and its
 * documentation for any purpose is hereby granted without fee, provided that
 * the above copyright notice appear in all copies and that both that
 * copyright notice and this permission notice appear in supporting
 * documentation, and that the name of Keith Packard not be used in
 * advertising or publicity pertaining to distribution of the software without
 * specific, written prior permission.  Keith Packard makes no
 * representations about the suitability of this software for any purpose.  It
 * is provided "as is" without express or implied warranty.
 *
 * KEITH PACKARD DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
 * INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS, IN NO
 * EVENT SHALL KEITH PACKARD BE LIABLE FOR ANY SPECIAL, INDIRECT OR
 * CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE,
 * DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
 * TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

/* Modified by Matthew Hawn. I don't know what to say here so follow what it
   says above. Not that I can really do anything about it
*/

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/extensions/Xcomposite.h>
#include <X11/extensions/Xdamage.h>
#include <X11/extensions/Xrender.h>
#include <X11/extensions/shape.h>
#include <getopt.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/poll.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

void draw_diagonals(Display *dpy, int scr, Window window) {
  unsigned long mask = 0;
  XGCValues values;
  values.foreground = XBlackPixel(dpy, scr);
  mask |= GCForeground;

  values.line_width = 10;
  mask |= GCLineWidth;

  GC gc = XCreateGC(dpy, window, mask, &values);
  XDrawLine(dpy, window, gc, 100, 100, 600, 600);
  XFlush(dpy);
  XFreeGC(dpy, gc);
}

int main(int argc, char **argv) {
  Display *dpy;
  XEvent ev;
  Window root_return, parent_return;
  Window *children;
  unsigned int nchildren;
  int i;
  XRenderPictureAttributes pa;
  XRectangle *expose_rects = NULL;
  int size_expose = 0;
  int n_expose = 0;
  struct pollfd ufd;
  int p;
  int composite_major, composite_minor;
  char *display = NULL;
  int o;

  dpy = XOpenDisplay(display);
  if (!dpy) {
    fprintf(stderr, "Can't open display\n");
    exit(1);
  }

  int scr = DefaultScreen(dpy);
  Window root = RootWindow(dpy, scr);

  Window overlay_window = XCompositeGetOverlayWindow(dpy, root);
  // pass input through overlay window
  XserverRegion overlay_region = XFixesCreateRegion(dpy, NULL, 0);
  XFixesSetWindowShapeRegion(dpy, overlay_window, ShapeInput, 0, 0,
                             overlay_region);
  XFixesDestroyRegion(dpy, overlay_region);
  draw_diagonals(dpy, scr, overlay_window);

  for (;;) {
    XNextEvent(dpy, &ev);
  }
}
