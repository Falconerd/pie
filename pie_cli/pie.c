#include <stdio.h>
#include <stdlib.h>
#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"
#include "../pie.h"

#define KB (1024ULL)
#define MB (KB * 1024)
#define MEMORY_SIZE (100 * MB)

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

size_t get_file_size(const char *filename) {
    FILE *file = fopen(filename, "rb");
    if (file == NULL) {
        perror("Error opening file");
        return -1;
    }

    fseek(file, 0, SEEK_END);
    size_t size = (size_t)ftell(file);
    fclose(file);

    return size;
}

size_t read_file_to_buffer(const char *filename, void *buf, size_t bufsize) {
    FILE *file = fopen(filename, "rb");
    if (file == NULL) {
        printf("Error opening file %s\n", filename);
        return 0;
    }

    fseek(file, 0, SEEK_END);
    size_t size = (size_t)ftell(file);
    if (size > bufsize) {
        printf("Buffer too small for file %s. Need %zd got %zd.\n",
               filename, size, bufsize);
        return 0;
    }

    rewind(file);

    size_t read_size = fread(buf, 1, size, file);
    if (read_size != size) {
        printf("Error reading file. Read size: %zd. size: %zd\n", read_size, size);
        fclose(file);
        return 0;
    }

    fclose(file);

    return read_size;
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

    memory = malloc(MEMORY_SIZE);

    int encode = 1;
    char *extension = argv[1] + strlen(argv[1]) - 4;
    if (memcmp(extension, ".pie", 4) == 0) {
        encode = 0;
    }

    size_t original_size = get_file_size(argv[1]);
    pie_size written; 
    
    if (encode) {
        int x, y, n;
        unsigned char *data = stbi_load(argv[1], &x, &y, &n, 0);
        if (!data) {
            printf("Could not load image file.");
            return -1;
        }

        pie_u16 w = (pie_u16)x;
        pie_u16 h = (pie_u16)y;

        pie_encoded encoded = pie_encode(data, w, h, 1, n, memory, MEMORY_SIZE);
        if (encoded.error) {
            printf("%s\n", pie_errors[encoded.error]);
            return -1;
        }

        FILE *fp = fopen(argv[2], "wb");
        if (!fp) {
            printf("Error opening file to write.\n");
            return -1;
        }
        written = (pie_size)fwrite(encoded.data, 1, encoded.size, fp);
        if (written != encoded.size) {
            printf("Error writing file.\n");
            return -1;
        }

        fclose(fp);
    } else {
        char *data = memory;
        size_t size = read_file_to_buffer(argv[1], data, MEMORY_SIZE);
        memory += size;
        pie_pixels decoded = pie_decode(data, memory, MEMORY_SIZE - size);
        if (decoded.error) {
            printf("%s\n", pie_errors[decoded.error]);
            return -1;
        }
        int w = decoded.width;
        int h = decoded.height;
        int comp = 4;
        int stride = w * comp;
        
        int success = stbi_write_png(argv[2], w, h, comp, memory, stride);
        if (!success) {
            printf("Failed to write image buffer.");
            return -1;
        }

        written = get_file_size(argv[2]);
    }

    if (written > original_size) {
        printf("Success. But, the resulting image is larger. %zdB -> %zdB\n",
               original_size, written);
    } else {
        printf("Success. %zdB -> %zdB\n", original_size, written);
    }

    return 0;
}

