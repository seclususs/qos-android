#include "daemon/config.hpp"
#include "daemon/parser.hpp"

namespace qos::config
{
    namespace
    {

        [[nodiscard]] constexpr bool is_true(std::string_view val) noexcept
        {
            return val == "1" || val == "true" || val == "True";
        }

    } // namespace

    ServiceState Loader::load_from_file(std::string_view path)
    {
        ServiceState state;
        const auto raw_config = qos::parser::Ini::parse_file(path);

        const auto fetch = [&raw_config](const char* key) -> bool
        {
            const auto it = raw_config.find(key);
            return it != raw_config.end() && is_true(it->second);
        };

        state.cpu = fetch("cpu_enabled");
        state.io = fetch("storage_enabled");
        state.cleaner = fetch("cleaner_enabled");
        state.tweaks = fetch("tweaks_enabled");
        state.blocker = fetch("blocker_enabled");

        return state;
    }

} // namespace qos::config