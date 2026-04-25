#include <openssl/evp.h>
#include <openssl/param_build.h>
#include <openssl/core_names.h>
#include "foundation/allocator.h"
#include "foundation/mem.h"
#include <stdio.h>
#include <string.h>

void handle_errors() {
    printf("An error occurred in OpenSSL.\n");
}

int main() {
    // 0. Initialize project memory tracking (50% RAM budget)
    cbm_mem_init(0, 0.5);

    EVP_PKEY_CTX *ctx = NULL;
    EVP_PKEY *pkey = NULL;
    OSSL_PARAM_BLD *pbld = NULL;
    OSSL_PARAM *params = NULL;
    unsigned char seed[32];
    memset(seed, 'B', 32);

    printf("Starting granular ML-DSA-44 seed test with project memory tracking...\n");

    // 1. Create context for ML-DSA-44
    ctx = EVP_PKEY_CTX_new_from_name(NULL, "ML-DSA-44", NULL);
    if (ctx == NULL) { handle_errors(); return 1; }

    // 2. Build parameters with the seed
    pbld = OSSL_PARAM_BLD_new();
    if (pbld == NULL) { handle_errors(); return 1; }
    
    if (OSSL_PARAM_BLD_push_octet_string(pbld, OSSL_PKEY_PARAM_ML_DSA_SEED, seed, 32) != 1) {
        handle_errors(); return 1;
    }
    
    params = OSSL_PARAM_BLD_to_param(pbld);
    if (params == NULL) { handle_errors(); return 1; }

    // 3. Generate key from seed data
    if (EVP_PKEY_fromdata_init(ctx) <= 0) { handle_errors(); return 1; }
    if (EVP_PKEY_fromdata(ctx, &pkey, EVP_PKEY_KEYPAIR, params) <= 0) {
        handle_errors(); return 1;
    }

    printf("Successfully generated ML-DSA-44 key from seed!\n");

    // 4. Test Signing
    EVP_MD_CTX *sctx = EVP_MD_CTX_new();
    const char *msg = "Test message for PQC signing via OpenSSL 4.0.0";
    size_t siglen = 0;
    unsigned char *sig = NULL;

    if (EVP_DigestSignInit_ex(sctx, NULL, NULL, NULL, NULL, pkey, NULL) <= 0) {
        handle_errors(); return 1;
    }

    if (EVP_DigestSign(sctx, NULL, &siglen, (unsigned char*)msg, strlen(msg)) <= 0) {
        handle_errors(); return 1;
    }

    // Use project-safe allocator
    sig = CBM_MALLOC(siglen);
    if (sig == NULL) { printf("Memory allocation failed\n"); return 1; }

    if (EVP_DigestSign(sctx, sig, &siglen, (unsigned char*)msg, strlen(msg)) <= 0) {
        handle_errors(); return 1;
    }

    printf("Successfully signed message! Signature length: %zu\n", siglen);

    // 5. Test Verification
    EVP_MD_CTX *vctx = EVP_MD_CTX_new();
    if (EVP_DigestVerifyInit_ex(vctx, NULL, NULL, NULL, NULL, pkey, NULL) <= 0) {
        handle_errors(); return 1;
    }

    if (EVP_DigestVerify(vctx, sig, siglen, (unsigned char*)msg, strlen(msg)) <= 0) {
        printf("Verification failed!\n");
    } else {
        printf("Verification successful!\n");
    }

    // Cleanup
    CBM_FREE(sig);
    EVP_MD_CTX_free(sctx);
    EVP_MD_CTX_free(vctx);
    EVP_PKEY_free(pkey);
    OSSL_PARAM_free(params);
    OSSL_PARAM_BLD_free(pbld);
    EVP_PKEY_CTX_free(ctx);

    return 0;
}
