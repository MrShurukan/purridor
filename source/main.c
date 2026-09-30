#include <switch.h>

extern int rust_main(void);

int main(int argc, char* argv[])
{
    Result rc = romfsInit();

    if (R_FAILED(rc))
    {
        return 1;
    }

    int result = rust_main();

    romfsExit();

    return result;
}