#ifndef REFRESH_MANAGER_H
#define REFRESH_MANAGER_H

#include <stdbool.h>

typedef struct refresh_manager refresh_manager_t;

refresh_manager_t *refresh_manager_create(const char *touch_dev_path);
void refresh_manager_start(refresh_manager_t *mgr);
void refresh_manager_stop(refresh_manager_t *mgr);
void refresh_manager_destroy(refresh_manager_t *mgr);

#endif /* REFRESH_MANAGER_H */
