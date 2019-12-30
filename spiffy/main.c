
// LEDs are located on GPIOD pin 12 though 15

// #include <stm32f4xx.h>

typedef unsigned int uint32_t;

volatile int a;
volatile int b;

void putc2(char c) {
    volatile uint32_t* ITM_STIM0 = 0xE0000000;
    while ((*ITM_STIM0) == 0) {}
    *ITM_STIM0 = c;
}

int main() {
    a = -500;
    while(1) {
        if (a == 0)
        {
            // putc2('A');
            // putc2('B');
            // putc2('C');
        }

        if (a > 1000) {
            a = -500;
        } else {
            a++;
        }

        // poor man delay:
        int i,j;
        for (i=0;i<100;i++) {
            for (j=0;j<100;j++)
            {
                // nop..
                b = i + j;
            }
        }
    }
    return 0;
}

