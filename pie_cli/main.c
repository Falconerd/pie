#include <stdio.h>
#include <stdlib.h>
#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include "../pie.h"

void *test_alloc(pie_size s, void *c) {
    (void)c;
    return malloc(s);
}

void test_free(pie_size s, void *p, void *c) {
    (void)s;
    (void)c;
    free(p);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Must supply an input file.");
    }
    if (argc < 3) {
        printf("Must supply an output file.");
    }

    pie_allocator a = {
        .alloc = test_alloc,
        .free = test_free
    };

    int x, y, n;
    unsigned char *data = stbi_load(argv[1], &x, &y, &n, 0);
    void *buffer = malloc(x * y * n);
    pie_bytes encoded = pie_encode(x, y, n == 4, 1, data, 0, &a);

    FILE *fp = fopen(argv[2], "wb");
    if (!fp) {
        printf("Error.\n");
        return -1;
    }
    size_t written = fwrite(encoded.data, 1, encoded.size, fp);
    if (written != encoded.size) {
        printf("Error.\n");
        return -1;
    }

    fclose(fp);

    return 0;
}
// typedef struct {
//     pie_u8 magic[3];       // 3
//     pie_u8 version;        // 1
//     pie_u32 flags;         // 4
//     pie_u16 width;         // 2
//     pie_u16 height;        // 2
//     pie_u32 length;        // 4
// } pie_header;              // 16
