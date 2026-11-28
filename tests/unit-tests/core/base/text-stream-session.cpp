#include <catch2/catch_test_macros.hpp>

#include "TextStream.h"
#include "SysInitIntf.h"
#include "tjsError.h"

#include <chrono>
#include <filesystem>
#include <fstream>
#include <memory>
#include <string>
#include <string_view>

namespace {
namespace fs = std::filesystem;

class TextFixtures {
public:
    TextFixtures() : directory(fs::temp_directory_path() /
        ("kiri-text-stream-session-" + std::to_string(
            std::chrono::steady_clock::now().time_since_epoch().count()))) {
        REQUIRE(fs::create_directory(directory));
    }

    ~TextFixtures() {
        std::error_code error;
        fs::remove_all(directory, error);
    }

    fs::path write(const char *name, std::string_view bytes) const {
        const auto path = directory / name;
        std::ofstream file(path, std::ios::binary);
        file.write(bytes.data(), bytes.size());
        file.close();
        REQUIRE(file.good());
        return path;
    }

private:
    const fs::path directory;
};

ttstr read_text(const fs::path &path) {
    const auto name = path.u8string();
    std::unique_ptr<iTJSTextReadStream> stream(
        TVPCreateTextStreamForRead(ttstr(name.c_str()), TJS_W("")));
    ttstr text;
    stream->Read(text, 0);
    return text;
}
} // namespace

TEST_CASE("Embedded text stream sessions reset script overrides") {
    const TextFixtures fixtures;
    const auto utf8 = fixtures.write("first.tjs", "\xE3\x81\x82 ");
    const auto cp932 = fixtures.write("second.tjs", "\x82\xA0\xFA\x40");

    TVPBeginEmbeddedSystemSession();
    TVPSetDefaultReadEncoding(TJS_W("utf8"));
    REQUIRE(ttstr(TVPGetDefaultReadEncoding()) == TJS_W("utf8"));
    REQUIRE(read_text(utf8) == TJS_W("\u3042 "));
    REQUIRE_THROWS_AS(read_text(cp932), TJS::eTJSError);

    TVPBeginEmbeddedSystemSession();
    CHECK(read_text(cp932) == TJS_W("\u3042\u2170"));
    CHECK(ttstr(TVPGetDefaultReadEncoding()) == TJS_W("UTF-8"));
}
