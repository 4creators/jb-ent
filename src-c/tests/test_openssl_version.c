#include <openssl/crypto.h>
#include <stdio.h>

int main() {
    unsigned int comp_major = OPENSSL_VERSION_MAJOR;
    unsigned int comp_minor = OPENSSL_VERSION_MINOR;
    unsigned int comp_patch = OPENSSL_VERSION_PATCH;

    unsigned int run_major = OPENSSL_version_major();
    unsigned int run_minor = OPENSSL_version_minor();
    unsigned int run_patch = OPENSSL_version_patch();

    printf("Compiled against OpenSSL %u.%u.%u\n", comp_major, comp_minor, comp_patch);
    printf("Linked at runtime to OpenSSL %u.%u.%u\n", run_major, run_minor, run_patch);
    printf("Full runtime string: %s\n", OpenSSL_version(OPENSSL_VERSION));

    if (comp_major != run_major || comp_minor != run_minor || comp_patch != run_patch) {
        fprintf(stderr, "FATAL ERROR: OpenSSL version mismatch. Loaded DLL/so version %u.%u.%u does not match header version %u.%u.%u.\n", run_major, run_minor, run_patch, comp_major, comp_minor, comp_patch);
        return 1;
    }

    printf("OpenSSL versions match %u.%u.%u.\n", comp_major, comp_minor, comp_patch);
    return 0;
}