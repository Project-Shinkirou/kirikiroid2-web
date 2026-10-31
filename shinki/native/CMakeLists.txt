if(NOT (WINDOWS OR LINUX OR APPLE OR ANDROID))
    message(FATAL_ERROR "KRKR2_SHINKI_EMU requires a desktop POSIX, Apple or Android target")
endif()
if(IOS OR KRKR2_SHINKI_EMU_STATIC)
    add_library(shinki_emu_kirikiroid2_web STATIC NativeOwner.cpp NativeSession.cpp NativeInput.cpp)
else()
    add_library(shinki_emu_kirikiroid2_web SHARED NativeOwner.cpp NativeSession.cpp NativeInput.cpp)
endif()
target_include_directories(shinki_emu_kirikiroid2_web PRIVATE
    "$<TARGET_PROPERTY:krkr2,INCLUDE_DIRECTORIES>"
    "${CMAKE_CURRENT_SOURCE_DIR}/include")
target_compile_definitions(shinki_emu_kirikiroid2_web PRIVATE
    "$<TARGET_PROPERTY:krkr2,COMPILE_DEFINITIONS>")
if(KRKR2_SHINKI_EMU_STATIC)
    target_compile_definitions(shinki_emu_kirikiroid2_web PRIVATE SHINKI_EMU_STATIC_CORE=1)
    target_sources(shinki_emu_kirikiroid2_web PRIVATE AndroidHost.cpp)
    # An unbuilt shared target exposes CMake's fully resolved transitive link
    # plan through the File API. The actual adapter and dependencies are static.
    add_library(shinki_krkr_link_plan SHARED NativeOwner.cpp NativeSession.cpp NativeInput.cpp)
    target_link_libraries(shinki_krkr_link_plan PRIVATE
        "$<TARGET_PROPERTY:krkr2,LINK_LIBRARIES>" EGL GLESv2)
endif()
target_compile_options(shinki_emu_kirikiroid2_web PRIVATE
    "$<TARGET_PROPERTY:krkr2,COMPILE_OPTIONS>")
target_link_libraries(shinki_emu_kirikiroid2_web PRIVATE
    "$<TARGET_PROPERTY:krkr2,LINK_LIBRARIES>")
target_link_options(shinki_emu_kirikiroid2_web PRIVATE
    "$<TARGET_PROPERTY:krkr2,LINK_OPTIONS>")
if(ANDROID)
    target_link_libraries(shinki_emu_kirikiroid2_web PRIVATE EGL GLESv2)
endif()
set_target_properties(shinki_emu_kirikiroid2_web PROPERTIES
    OUTPUT_NAME "shinki_emu_kirikiroid2_web"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/$<0:>"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/$<0:>"
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/$<0:>")
if(UNIX AND NOT APPLE)
    target_link_libraries(shinki_emu_kirikiroid2_web PRIVATE ${CMAKE_DL_LIBS})
    set_target_properties(shinki_emu_kirikiroid2_web PROPERTIES
        BUILD_WITH_INSTALL_RPATH TRUE INSTALL_RPATH "$ORIGIN")
elseif(APPLE)
    set_target_properties(shinki_emu_kirikiroid2_web PROPERTIES INSTALL_RPATH "@loader_path")
endif()
if(WINDOWS)
    cocos_copy_target_dll(shinki_emu_kirikiroid2_web)
endif()
set(GAME_RES_FOLDER "${PROJECT_SOURCE_DIR}/ui/cocos-studio")
add_custom_target(shinki_emu_kirikiroid2_web_resources
    COMMAND ${CMAKE_COMMAND} -E copy_directory
        "${GAME_RES_FOLDER}" "$<TARGET_FILE_DIR:shinki_emu_kirikiroid2_web>/Resources"
    VERBATIM)
add_dependencies(shinki_emu_kirikiroid2_web shinki_emu_kirikiroid2_web_resources)
# Test-only static library: C entry points over NativeLifecycle.h for the Rust
# crate's unit tests. It is never linked into the core DLL; it lands next to the
# import library, where the crate's link search finds it.
add_library(shinki_emu_kirikiroid2_web_lifecycle_test STATIC NativeLifecycleTest.cpp)
target_include_directories(shinki_emu_kirikiroid2_web_lifecycle_test PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/include")
target_compile_features(shinki_emu_kirikiroid2_web_lifecycle_test PRIVATE cxx_std_17)
set_target_properties(shinki_emu_kirikiroid2_web_lifecycle_test PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/$<0:>")
add_dependencies(shinki_emu_kirikiroid2_web shinki_emu_kirikiroid2_web_lifecycle_test)
