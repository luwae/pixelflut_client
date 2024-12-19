#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>

#include "../common.h"

#define LOADED_FILE_SIZE (256*1024ull)
void *loaded_file = MAP_FAILED;

unsigned long long refcnt = 0;

void global_init() {
    if (refcnt > 0)
        return;

    int fd = open("res/fs.img", O_RDONLY);
    EASSERT(fd != -1, "open");
    loaded_file = mmap(NULL, LOADED_FILE_SIZE, PROT_READ, MAP_PRIVATE, fd, 0);
    EASSERT(loaded_file != MAP_FAILED, "mmap");
    close(fd);
}

struct file_iter {
    unsigned int x, y;
};

void *iter_create(void *arg) {
    (void) arg;
    global_init();
    if (loaded_file == MAP_FAILED) { // catch init errors
        return NULL;
    }

    struct file_iter *it = malloc(sizeof(*it));
    if (it != NULL) {
        it->x = it->y = 0;
        refcnt++;
    }
    return it;
}

void iter_destroy(void *it) {
    free(it);
}

int iter_next(void *it_raw, struct px *px) {
    struct file_iter *it = it_raw;
    px->x = it->x;
    px->y = it->y;
    unsigned char col = ((char*)loaded_file)[it->x + 512 * it->y];
    px->r = px->g = px->b = col;
    // advance
    it->x += 1;
    if (it->x == 512) {
        it->x = 0;
        it->y += 1;
        if (it->y == 512) {
            // start at the front again
            it->y = 0;
        }
    }
    return 1;
}
