#pragma once

namespace qos::core
{

    class App
    {
        public:
        [[nodiscard]] static int bootstrap() noexcept;
    };

} // namespace qos::core