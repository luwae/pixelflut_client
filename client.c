#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <string.h>

int aton_ipv4(const char *ip) {
    int parts[4];
    if (sscanf(ip, "%d.%d.%d.%d", parts, parts + 1, parts + 2, parts + 3) != 4) {
        perror("scanf");
        exit(1);
    }
    for (int i = 0; i < 4; i++) {
        if (parts[i] < 0 || parts[i] > 255) {
            printf("invalid ipv4\n");
            exit(1);
        }
    }
    return htonl((parts[0]<<24) | (parts[1]<<16) | (parts[2]<<8) | parts[3]);
}

struct pixel {
    unsigned int x;
    unsigned int y;
    unsigned char r;
    unsigned char g;
    unsigned char b;
};

char PIXEL[100];
void pixel_write(const struct pixel *px, int fd) {
    int len = sprintf(PIXEL, "PX %d %d %02x%02x%02x\n",
            px->x, px->y, px->r, px->g, px->b);
    int n = write(fd, (void *)PIXEL, len);
    if (n == -1) {
        perror("write");
        exit(1);
    } else if (n != len) {
        printf("not all bytes transferred\n");
    }
}

int main() {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd == -1) {
        perror("socket");
        exit(1);
    }

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(8000),
        .sin_addr = aton_ipv4("193.196.38.83")
    };
    int status = connect(fd, (struct sockaddr *)&addr, sizeof(addr));
    if (status == -1) {
        perror("connect");
        exit(1);
    }

    struct pixel px = { .x = 0, .y = 0, .r = 255, .g = 0, .b = 0 };
    pixel_write(&px, fd);

    close(fd);
}
