#ifndef KNAVE_SHELL_CORE_H
#define KNAVE_SHELL_CORE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct KsCore KsCore;

KsCore *ks_core_new(void);
void ks_core_free(KsCore *core);

bool ks_core_refresh(KsCore *core);
bool ks_core_connected(const KsCore *core);
const char *ks_core_last_error(const KsCore *core);
uint64_t ks_core_preview_revision(const KsCore *core);

size_t ks_core_workspace_count(const KsCore *core);
uint32_t ks_core_workspace_number(const KsCore *core, size_t index);
bool ks_core_workspace_active(const KsCore *core, size_t index);
size_t ks_core_workspace_window_count(const KsCore *core, size_t index);
size_t ks_core_workspace_visible_window_count(const KsCore *core, size_t index);

size_t ks_core_window_count(const KsCore *core);
uint64_t ks_core_window_id(const KsCore *core, size_t index);
const char *ks_core_window_title(const KsCore *core, size_t index);
const char *ks_core_window_app_id(const KsCore *core, size_t index);
uint32_t ks_core_window_workspace(const KsCore *core, size_t index);
bool ks_core_window_minimized(const KsCore *core, size_t index);
bool ks_core_window_floating(const KsCore *core, size_t index);
bool ks_core_window_fullscreen(const KsCore *core, size_t index);
bool ks_core_window_focused(const KsCore *core, size_t index);

const uint8_t *ks_core_preview_png(
    const KsCore *core,
    uint32_t workspace,
    size_t *length
);

bool ks_core_focus_workspace(KsCore *core, uint32_t workspace);
bool ks_core_focus_window(KsCore *core, uint64_t window);

#ifdef __cplusplus
}
#endif

#endif

