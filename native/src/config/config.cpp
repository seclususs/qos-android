#include "daemon/config.hpp"
#include "daemon/logger.hpp"
#include "daemon/parser.hpp"

#include <algorithm>
#include <charconv>
#include <string>

namespace qos::config
{
    namespace
    {

        [[nodiscard]] constexpr bool is_true(std::string_view val) noexcept
        {
            return val == "1" || val == "true" || val == "True";
        }

        [[nodiscard]] uint64_t parse_u64(const std::string& val, uint64_t default_val) noexcept
        {
            uint64_t result = 0;
            auto [ptr, ec] = std::from_chars(val.data(), val.data() + val.size(), result);

            if (ec == std::errc() && ptr == val.data() + val.size())
            {
                return result;
            }

            return default_val;
        }

    } // namespace

    ServiceState Loader::load_from_file(std::string_view path)
    {
        ServiceState state;
        const auto raw_config = qos::parser::Ini::parse_file(path);

        const auto fetch_bool = [&raw_config](const char* key) -> bool
        {
            const auto it = raw_config.find(key);
            return it != raw_config.end() && is_true(it->second);
        };

        const auto fetch_u64 = [&raw_config](const char* key, uint64_t def) -> uint64_t
        {
            const auto it = raw_config.find(key);

            if (it != raw_config.end())
            {
                if (it->second.empty())
                    return EMPTY_U64;

                return parse_u64(it->second, def);
            }

            return def;
        };

        const auto has_key = [&raw_config](const char* key) -> bool
        { return raw_config.find(key) != raw_config.end(); };

        state.cpu = fetch_bool("cpu_enabled");
        state.io = fetch_bool("storage_enabled");
        state.cleaner = fetch_bool("cleaner_enabled");
        state.tweaks = fetch_bool("tweaks_enabled");
        state.blocker = fetch_bool("blocker_enabled");

        if (has_key("min_latency_ns") || has_key("max_latency_ns") || has_key("min_granularity_ns") ||
            has_key("max_granularity_ns") || has_key("min_wakeup_ns") || has_key("max_wakeup_ns") ||
            has_key("min_migration_cost") || has_key("max_migration_cost") || has_key("min_walt_init_pct") ||
            has_key("max_walt_init_pct") || has_key("min_uclamp_min") || has_key("max_uclamp_min"))
        {
            state.has_cpu_limits = true;
            state.cpu_limits.min_latency_ns = fetch_u64("min_latency_ns", state.cpu_limits.min_latency_ns);
            state.cpu_limits.max_latency_ns = fetch_u64("max_latency_ns", state.cpu_limits.max_latency_ns);
            state.cpu_limits.min_granularity_ns = fetch_u64("min_granularity_ns", state.cpu_limits.min_granularity_ns);
            state.cpu_limits.max_granularity_ns = fetch_u64("max_granularity_ns", state.cpu_limits.max_granularity_ns);
            state.cpu_limits.min_wakeup_ns = fetch_u64("min_wakeup_ns", state.cpu_limits.min_wakeup_ns);
            state.cpu_limits.max_wakeup_ns = fetch_u64("max_wakeup_ns", state.cpu_limits.max_wakeup_ns);
            state.cpu_limits.min_migration_cost = fetch_u64("min_migration_cost", state.cpu_limits.min_migration_cost);
            state.cpu_limits.max_migration_cost = fetch_u64("max_migration_cost", state.cpu_limits.max_migration_cost);
            state.cpu_limits.min_walt_init_pct = fetch_u64("min_walt_init_pct", state.cpu_limits.min_walt_init_pct);
            state.cpu_limits.max_walt_init_pct = fetch_u64("max_walt_init_pct", state.cpu_limits.max_walt_init_pct);
            state.cpu_limits.min_uclamp_min = fetch_u64("min_uclamp_min", state.cpu_limits.min_uclamp_min);
            state.cpu_limits.max_uclamp_min = fetch_u64("max_uclamp_min", state.cpu_limits.max_uclamp_min);
        }

        if (has_key("min_read_ahead") || has_key("max_read_ahead") || has_key("min_nr_requests") ||
            has_key("max_nr_requests"))
        {
            state.has_storage_limits = true;
            state.storage_limits.min_read_ahead = fetch_u64("min_read_ahead", state.storage_limits.min_read_ahead);
            state.storage_limits.max_read_ahead = fetch_u64("max_read_ahead", state.storage_limits.max_read_ahead);
            state.storage_limits.min_nr_requests = fetch_u64("min_nr_requests", state.storage_limits.min_nr_requests);
            state.storage_limits.max_nr_requests = fetch_u64("max_nr_requests", state.storage_limits.max_nr_requests);
        }

        if (state.has_cpu_limits)
        {
            auto& cpu = state.cpu_limits;

            auto safe_swap = [](uint64_t& a, uint64_t& b)
            {
                if (a != EMPTY_U64 && b != EMPTY_U64 && a > b)
                    std::swap(a, b);
            };

            safe_swap(cpu.min_latency_ns, cpu.max_latency_ns);
            safe_swap(cpu.min_granularity_ns, cpu.max_granularity_ns);
            safe_swap(cpu.min_wakeup_ns, cpu.max_wakeup_ns);
            safe_swap(cpu.min_migration_cost, cpu.max_migration_cost);
            safe_swap(cpu.min_walt_init_pct, cpu.max_walt_init_pct);
            safe_swap(cpu.min_uclamp_min, cpu.max_uclamp_min);

            auto safe_clamp = [](uint64_t& val, uint64_t min_limit, uint64_t max_limit)
            {
                if (val != EMPTY_U64)
                    val = std::clamp(val, min_limit, max_limit);
            };

            safe_clamp(cpu.min_latency_ns, 1000000UL, 100000000UL);
            safe_clamp(cpu.max_latency_ns, 1000000UL, 100000000UL);
            safe_clamp(cpu.min_granularity_ns, 500000UL, 50000000UL);
            safe_clamp(cpu.max_granularity_ns, 500000UL, 50000000UL);
            safe_clamp(cpu.min_wakeup_ns, 500000UL, 50000000UL);
            safe_clamp(cpu.max_wakeup_ns, 500000UL, 50000000UL);
            safe_clamp(cpu.min_migration_cost, 50000UL, 5000000UL);
            safe_clamp(cpu.max_migration_cost, 50000UL, 5000000UL);
            safe_clamp(cpu.min_walt_init_pct, 1UL, 100UL);
            safe_clamp(cpu.max_walt_init_pct, 1UL, 100UL);
            safe_clamp(cpu.min_uclamp_min, 0UL, 1024UL);
            safe_clamp(cpu.max_uclamp_min, 0UL, 1024UL);
        }

        if (state.has_storage_limits)
        {
            auto& io = state.storage_limits;

            auto safe_swap = [](uint64_t& a, uint64_t& b)
            {
                if (a != EMPTY_U64 && b != EMPTY_U64 && a > b)
                    std::swap(a, b);
            };

            safe_swap(io.min_read_ahead, io.max_read_ahead);
            safe_swap(io.min_nr_requests, io.max_nr_requests);

            auto safe_clamp = [](uint64_t& val, uint64_t min_limit, uint64_t max_limit)
            {
                if (val != EMPTY_U64)
                    val = std::clamp(val, min_limit, max_limit);
            };

            safe_clamp(io.min_read_ahead, 16UL, 4096UL);
            safe_clamp(io.max_read_ahead, 16UL, 4096UL);
            safe_clamp(io.min_nr_requests, 8UL, 1024UL);
            safe_clamp(io.max_nr_requests, 8UL, 1024UL);
        }

        LOGD("Config: cpu=%d io=%d cleaner=%d tweaks=%d blocker=%d", state.cpu, state.io, state.cleaner, state.tweaks,
             state.blocker);

        if (state.has_cpu_limits)
        {
            LOGD("CPU Limits: lat[%lu,%lu] gran[%lu,%lu] wake[%lu,%lu] mig[%lu,%lu] walt[%lu,%lu] uclamp[%lu,%lu]",
                 state.cpu_limits.min_latency_ns, state.cpu_limits.max_latency_ns, state.cpu_limits.min_granularity_ns,
                 state.cpu_limits.max_granularity_ns, state.cpu_limits.min_wakeup_ns, state.cpu_limits.max_wakeup_ns,
                 state.cpu_limits.min_migration_cost, state.cpu_limits.max_migration_cost,
                 state.cpu_limits.min_walt_init_pct, state.cpu_limits.max_walt_init_pct,
                 state.cpu_limits.min_uclamp_min, state.cpu_limits.max_uclamp_min);
        }

        if (state.has_storage_limits)
        {
            LOGD("Storage Limits: read_ahead[%lu,%lu] nr_requests[%lu,%lu]", state.storage_limits.min_read_ahead,
                 state.storage_limits.max_read_ahead, state.storage_limits.min_nr_requests,
                 state.storage_limits.max_nr_requests);
        }

        return state;
    }

} // namespace qos::config