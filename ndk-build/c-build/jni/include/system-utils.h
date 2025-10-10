#ifndef SYSTEM_UTILS_H
#define SYSTEM_UTILS_H

#include <stdbool.h>

bool sys_write_file(const char *path, const char *value);
void sys_set_property(const char *key, const char *value);
bool sys_set_refresh_rate_cmd(const char *rate_str);

#endif /* SYSTEM_UTILS_H */
