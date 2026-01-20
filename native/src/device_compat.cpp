// Author: [Seclususs](https://github.com/seclususs)

#include "device_compat.h"
#include "logging.h"

#include <cstring>
#include <sys/system_properties.h>

namespace qos::compat {

bool DeviceCompat::should_force_disable_display() {
  char device_prop[PROP_VALUE_MAX] = {0};
  char build_id_prop[PROP_VALUE_MAX] = {0};

  // Retrieve identifying system properties.
  int dev_len = __system_property_get("ro.product.device", device_prop);
  int build_len = __system_property_get("ro.build.id", build_id_prop);

  // If properties cannot be read, assume standard behavior.
  if (dev_len <= 0 || build_len <= 0) {
    return false;
  }

  // Check for specific target
  bool is_target_device = (std::strcmp(device_prop, "selene") == 0);

  // Check for specific build
  bool is_target_build =
      (std::strcmp(build_id_prop, "TQ3A.230901.001.B1") == 0);

  if (is_target_device && is_target_build) {
    LOGI("DeviceCompat: Known incompatible device detected.");
    return true;
  }

  return false;
}

} // namespace qos::compat