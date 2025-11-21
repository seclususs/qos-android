/**
 * @author Seclususs
 * https://github.com/seclususs
 */

#include "system_utils.h"
#include "fd_wrapper.h"
#include "logging.h"

#include <fstream>
#include <vector>
#include <cstring>
#include <cerrno>
#include <unistd.h>
#include <sys/system_properties.h>
#include <sys/wait.h>

namespace SystemUtils {

    bool applyTweak(const std::string& path, const std::string& value) {
        FdWrapper fd(path.c_str(), O_WRONLY | O_TRUNC);
        if (fd.isValid()) {
            ssize_t written = fd.write(value.c_str(), value.size());
            if (written == static_cast<ssize_t>(value.size())) {
                return true;
            }
            LOGD("Partial write to %s: %zd/%zu bytes", path.c_str(), written, value.size());
        }

        std::ofstream outfile(path);
        if (!outfile) {
            LOGE("Failed to open for writing: %s (errno: %d - %s)",
                 path.c_str(), errno, strerror(errno));
            return false;
        }

        outfile << value;
        if (outfile.fail()) {
            LOGE("Failed to write '%s' to: %s", value.c_str(), path.c_str());
            return false;
        }

        return true;
    }

    void setSystemProp(const std::string& key, const std::string& value) {
        if (__system_property_set(key.c_str(), value.c_str()) < 0) {
            LOGE("Failed to set system property: %s (errno: %d - %s)",
                 key.c_str(), errno, strerror(errno));
        }
    }

    bool setAndroidSetting(const std::string& property, const std::string& value) {
        std::vector<std::string> args = {
            "/system/bin/settings",
            "put",
            "system",
            property,
            value
        };

        std::vector<char*> argv;
        for (const auto& arg : args) {
            argv.push_back(const_cast<char*>(arg.c_str()));
        }
        argv.push_back(nullptr);

        int pipefd[2];
        if (pipe(pipefd) == -1) {
            LOGE("setAndroidSetting: pipe() failed (errno: %d - %s)",
                 errno, strerror(errno));
            return false;
        }

        pid_t pid = fork();
        if (pid == -1) {
            LOGE("setAndroidSetting: fork() failed (errno: %d - %s)",
                 errno, strerror(errno));
            close(pipefd[0]);
            close(pipefd[1]);
            return false;
        }

        if (pid == 0) {
            close(pipefd[0]);

            while ((dup2(pipefd[1], STDOUT_FILENO) == -1) && (errno == EINTR)) {}
            while ((dup2(pipefd[1], STDERR_FILENO) == -1) && (errno == EINTR)) {}

            close(pipefd[1]);
            execv(argv[0], argv.data());

            fprintf(stderr, "execv failed: %s\n", strerror(errno));
            _exit(127);

        } else {
            close(pipefd[1]);

            char buffer[128];
            std::string output;
            ssize_t count;

            while ((count = read(pipefd[0], buffer, sizeof(buffer) - 1)) > 0) {
                buffer[count] = '\0';
                output += buffer;
            }

            close(pipefd[0]);

            int status;
            waitpid(pid, &status, 0);

            int exit_code = WIFEXITED(status) ? WEXITSTATUS(status) : -1;

            if (exit_code == 0) {
                LOGI("Successfully set '%s' to %s", property.c_str(), value.c_str());
                return true;
            }

            if (!output.empty() && output.back() == '\n') {
                output.pop_back();
            }

            LOGE("Failed to set '%s' to %s. Code: %d, Output: %s",
                 property.c_str(), value.c_str(), exit_code, output.c_str());

            return false;
        }
    }

}