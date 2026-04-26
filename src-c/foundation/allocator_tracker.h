#ifndef CBM_ALLOCATOR_TRACKER_H
#define CBM_ALLOCATOR_TRACKER_H

#ifdef __cplusplus

#include <stddef.h>

#ifdef CBM_HARDEN_MEMORY

extern "C" void cbm_mem_print_audit(void);
extern "C" void cbm_set_tracker_memory(void* mem, size_t capacity);

// Construct this first so it destructs LAST, reporting memory
// only after all other global statics have been cleaned up.
struct CbmMemTrackerReporter {
    void* mem;
    CbmMemTrackerReporter() {
        size_t capacity = (1ULL << 26); // 67 million allocations
        // Allocate the tracker table using `new` (which maps to CBM_MALLOC).
        // This triggers tracker_add, which stores it in the delayed slot.
        mem = new char[capacity * 32](); // Zero-initialized
        cbm_set_tracker_memory(mem, capacity);
    }
    ~CbmMemTrackerReporter() {
        cbm_mem_print_audit();
        delete[] (char*)mem;
    }
};

static CbmMemTrackerReporter g_tracker_reporter;

#endif /* CBM_HARDEN_MEMORY */

#endif /* __cplusplus */

#endif /* CBM_ALLOCATOR_TRACKER_H */
