#include <stdio.h>
#include <stdlib.h>
#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include "../pie.h"

#define KB (1024ULL)
#define MB (KB * 1024)

pie_u8 *memory;
pie_size offset = 0;

void *test_alloc(pie_size s, void *c) {
    (void)c;
    pie_u8 *ptr = &memory[offset];
    offset += s;
    return ptr;
}

void test_free(pie_size s, void *p, void *c) {
    (void)s;
    (void)p;
    (void)c;
}

long get_file_size(const char *filename) {
    FILE *file = fopen(filename, "rb");
    if (file == NULL) {
        perror("Error opening file");
        return -1;
    }

    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    fclose(file);

    return size;
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Must supply an input file.");
        return -1;
    }
    if (argc < 3) {
        printf("Must supply an output file.");
        return -1;
    }

    pie_allocator a = {
        .alloc = test_alloc,
        .free = test_free
    };

    memory = malloc(100 * MB);

    int x, y, n;
    unsigned char *data = stbi_load(argv[1], &x, &y, &n, 0);
    if (!data) {
        printf("Could not load image file.");
        return -1;
    }

    long original_size = get_file_size(argv[1]);

    pie_bytes encoded = pie_encode((pie_u16)x, (pie_u16)y, n == 4, 1, data, &a);
    if (encoded.error_code) {
        printf("%s\n", pie_errors[encoded.error_code]);
        return -1;
    }

    FILE *fp = fopen(argv[2], "wb");
    if (!fp) {
        printf("Error.\n");
        return -1;
    }
    pie_size written = (pie_size)fwrite(encoded.data, 1, encoded.size, fp);
    if (written != encoded.size) {
        printf("Error.\n");
        return -1;
    }

    fclose(fp);

    if (written > original_size) {
        printf("Success. But, the resulting image is larger. %ldB -> %zdB\n",
               original_size, written);
    } else {
        printf("Success. %ldB -> %zdB\n", original_size, written);
    }

    return 0;
}

