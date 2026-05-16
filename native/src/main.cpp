// This file is part of QoS-Android.
// Licensed under the GNU GPL v3.
// Author: [Seclususs](https://github.com/seclususs)

#include "daemon/daemon.hpp"

int main([[maybe_unused]] int argc, [[maybe_unused]] char* argv[])
{
    return qos::core::App::bootstrap();
}