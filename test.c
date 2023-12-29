/* p
i
e
flags
width

height

data



*/






#include <assert.h>
#include <stdio.h>
#include "pie.h"

typedef unsigned long long pie_u64;

typedef struct {
    pie_u8 p;
    pie_u8 i;
    pie_u8 e;
    pie_u8 flags;
    pie_u16 width;
    pie_u16 height;
    void *data;
} test_struct;

pie_u8 bytes[] = {
    0x50, 0x49, 0x45, // PIE
    0x01, // Flags
    0x08, 0x00, // Width
    0x08, 0x00, // Height
    // RLE encoded data [Index, Run Length]
    // Add up the Run Lengths and it should equal Width * Height.
    // In this case 8x8 = 64.
    0x01, 0x01,
    0x00, 0x06,
    0x01, 0x06,
    0x02, 0x02,
    0x01, 0x01,
    0x02, 0x04,
    0x01, 0x01,
    0x02, 0x02,
    0x01, 0x01,
    0x02, 0x04,
    0x01, 0x01,
    0x02, 0x02,
    0x01, 0x06,
    0x02, 0x02,
    0x01, 0x01,
    0x02, 0x06,
    0x01, 0x02,
    0x02, 0x06,
    0x01, 0x01,
    0x02, 0x01,
    0x01, 0x01,
    0x03, 0x06,
    0x01, 0x02,
    // To determine the end of this block, a running total can be kept in the
    // decoder. When it equals the Width * Height, that's the end.

    // Highest index is 3, so 4 colours will be assumed for the palette.
    0x6A, 0xBE, 0x30,
    0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00,
    0x5B, 0x6E, 0xE1
};

int main(void) {
    pie_test_typesizes typesizes = pie_validate_types();
    if (!typesizes.pie_u8_correct) {
        printf("Expected pie_u8 size to be 1 byte. Got %d\n",
                typesizes.pie_u8_size);
    }
    if (!typesizes.pie_u16_correct) {
        printf("Expected pie_u16 size to be 2 bytes. Got %d\n",
                typesizes.pie_u16_size);
    }

    int size = sizeof(test_struct);
    printf("%d\n", size);

    pie_header h = *(pie_header *)bytes;

    __debugbreak();

    return 0;
}
