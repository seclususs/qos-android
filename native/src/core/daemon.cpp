#include "daemon/daemon.hpp"
#include "daemon/config.hpp"
#include "daemon/crash.hpp"
#include "daemon/detector.hpp"
#include "daemon/logger.hpp"
#include "daemon/tuner.hpp"
#include "qos/bridge.hpp"

#include <csignal>
#include <cstdlib>
#include <malloc.h>
#include <sys/signalfd.h>

#ifndef M_DECAY_TIME
#define M_DECAY_TIME -100
#endif

#ifndef M_PURGE
#define M_PURGE -101
#endif

namespace qos::core
{

    int App::bootstrap() noexcept
    {
        ::mallopt(M_DECAY_TIME, 0);
        LOGI("=== Daemon Starting ===");

        LOGI("Hardening Environment...");
        qos::core::Handler::arm();
        qos::system::Tuner::harden_process();
        qos::system::Tuner::expand_resources();
        qos::system::Tuner::enforce_efficiency_mode();
        qos::system::Tuner::set_realtime_policy();
        qos::system::Tuner::maximize_timer_slack();
        qos::system::Tuner::limit_cpu_utilization();
        qos::system::Tuner::set_high_io_priority();

        LOGI("Checking Hardware Support...");
        const auto features = qos::system::Detector::check_features();

        LOGI("Loading Configuration...");
        const auto cfg = config::Loader::load_from_file("/data/adb/modules/sys_qos/config.ini");

        const bool run_cpu = cfg.cpu && features.has_cpu_psi;

        const bool run_io = cfg.io && features.has_io_psi;

        const bool run_cleaner =
            cfg.cleaner && features.cleaner_supported && features.has_cpu_psi && features.has_io_psi;

        const bool run_tweaks = cfg.tweaks;

        const bool run_blocker = cfg.blocker;

        if (!run_cpu && !run_io && !run_cleaner && !run_tweaks && !run_blocker)
        {
            LOGE("Daemon shutting down (No services enabled).");
            return EXIT_FAILURE;
        }

        LOGI("Activating Services...");
        ::set_cpu_service(run_cpu);
        ::set_storage_service(run_io);
        ::set_cleaner_service(run_cleaner);
        ::set_tweaks(run_tweaks);
        ::set_blocker_service(run_blocker);

        if (cfg.has_cpu_limits)
        {
            ::set_cpu_limits(&cfg.cpu_limits);
        }

        if (cfg.has_storage_limits)
        {
            ::set_storage_limits(&cfg.storage_limits);
        }

        ::mallopt(M_PURGE, 0);
        qos::system::Tuner::lock_memory();

        sigset_t mask;
        ::sigemptyset(&mask);
        ::sigaddset(&mask, SIGINT);
        ::sigaddset(&mask, SIGTERM);
        ::sigaddset(&mask, SIGHUP);
        ::sigprocmask(SIG_BLOCK, &mask, nullptr);

        const int sfd = ::signalfd(-1, &mask, SFD_CLOEXEC | SFD_NONBLOCK);

        LOGI("Handover to Core Logic...");
        const int rust_status = ::start_services(sfd);

        if (rust_status != 0)
        {
            LOGE("Fatal: Core services failed to start (Error: %d).", rust_status);
            return EXIT_FAILURE;
        }

        LOGI("Core services running. Main thread waiting...");
        ::join_threads();

        LOGI("=== Shutdown Cleanly ===");
        return EXIT_SUCCESS;
    }

} // namespace qos::core