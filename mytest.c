#include <stdio.h>

#include "common.h"

int dummy;

void *iter_create(void) {
    return &dummy;
}

void iter_destroy(void *it) {
    // nothing
}

int iter_next(void *it, struct px *px) {
    if (dummy > 10) {
        return 0;
    }
    px->x = dummy;
    dummy++;
    return 1;
}
