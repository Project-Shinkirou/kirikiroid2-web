#pragma once
#ifndef SHINKI_EMU_STATIC_CORE
#include <filesystem>
#endif
namespace shinki::krkr::android {
#ifdef SHINKI_EMU_STATIC_CORE
bool initialized() noexcept;
#else
bool set_context(void *vm, void *application);
std::filesystem::path activate();
#endif
void owner_started();
void drain();
void owner_stopped();
}
