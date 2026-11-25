cmake_minimum_required(VERSION 3.16)
project(Tests LANGUAGES CXX)

find_package(Catch2 3 REQUIRED)

include(CTest)
include(Catch)

set(TEST_CONFIG_DIR "${CMAKE_CURRENT_BINARY_DIR}")

add_executable(narrow-text unit-tests/core/base/narrow-text.cpp
    ../cpp/core/utils/encoding/jis2unicode.c
    ../cpp/core/utils/encoding/gbk2unicode.c)
target_include_directories(narrow-text PRIVATE ../cpp/core/base)
target_link_libraries(narrow-text PRIVATE Catch2::Catch2WithMain)
catch_discover_tests(narrow-text)

add_subdirectory(unit-tests/core/movie)
add_subdirectory(unit-tests/core/visual)
add_subdirectory(unit-tests/core/tjs2)
add_subdirectory(unit-tests/core/base)

add_subdirectory(unit-tests/plugins)
add_subdirectory(differential/native)

set(TEST_FILES_PATH ${CMAKE_CURRENT_SOURCE_DIR}/test_files)
set(REFERENCE_PATH ${CMAKE_SOURCE_DIR}/reference)
configure_file(test_config.h.in test_config.h)
