#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "../common.h"

unsigned long long refcnt = 0;

void global_init() {
    if (refcnt > 0)
        return;

    srand(time(NULL));
}

int dummy;

void *iter_create(void *arg) {
    (void) arg;
    global_init();
    refcnt++;
    return &dummy;
}

void iter_destroy(void *it) {
    // nothing
}

int iter_next(void *it_raw, struct px *px) {
    (void) it_raw;
    int pos = rand();
    px->x = pos & 0x1ff;
    px->y = (pos >> 16) & 0x1ff;
    int color = rand();
    px->r = color & 0xff;
    px->g = color & 0xff;
    px->b = color & 0xff;
    return 1;
}
