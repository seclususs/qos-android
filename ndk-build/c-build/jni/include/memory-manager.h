#ifndef MEMORY_MANAGER_H
#define MEMORY_MANAGER_H

#include <stdbool.h>

typedef struct memory_manager memory_manager_t;

memory_manager_t *memory_manager_create(void);
void memory_manager_start(memory_manager_t *mgr);
void memory_manager_stop(memory_manager_t *mgr);
void memory_manager_destroy(memory_manager_t *mgr);

#endif /* MEMORY_MANAGER_H */
