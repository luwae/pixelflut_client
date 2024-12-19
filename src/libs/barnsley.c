#include <stdlib.h>
#include <time.h>

#include "../common.h"

void global_init() {
    static int initialized = 0;
    if (!initialized) {
        initialized = 1;
        srand(time(NULL));
    }
}

struct barnsley_iter {
  double x, y;
};

void *iter_create(void) {
    global_init();
    struct barnsley_iter *it = malloc(sizeof(*it));
    if (it != NULL) {
        it->x = it->y = 0;
    }
    return it;
}

void iter_destroy(void *it) {
    free(it);
}

int iter_next(void *it_raw, struct px *px) {
    struct barnsley_iter *it = it_raw;
    double xn, yn;
    double num = rand() / (double)RAND_MAX;
    if (num < 0.01) {
        xn = 0.0;
        yn = 0.16 * it->y;
    } else if (num < 0.86) {
        xn = 0.85 * it->x + 0.04 * it->y;
        yn = -0.04 * it->x + 0.85 * it->y + 1.6;
    } else if (num < 0.93) {
        xn = 0.2 * it->x - 0.26 * it->y;
        yn = 0.23 * it->x + 0.22 * it->y + 1.6;
    } else {
        xn = -0.15 * it->x + 0.28 * it->y;
        yn = 0.26 * it->x + 0.24 * it->y + 0.44;
    }
    it->x = xn;
    it->y = yn;

    px->x = (unsigned int)((it->x + 5.0) * 50.0);
    px->y = (unsigned int)(it->y * 50.0);
    px->r = 250 - (px->y >> 3);
    px->g = 50 + (px->y >> 2);
    px->b = 50 - (px->y >> 4);
    return 1;
}
