#include "layershellintegration.h"

#include <QCoreApplication>
#include <QtWaylandClient/private/qwaylandwindow_p.h>
#include <wayland-client-protocol.h>

KnaveLayerShellIntegration::KnaveLayerShellIntegration()
    : QWaylandShellIntegrationTemplate(1)
{
}

QtWaylandClient::QWaylandShellSurface *
KnaveLayerShellIntegration::createShellSurface(QtWaylandClient::QWaylandWindow *window)
{
    return new KnaveLayerSurface(this, window);
}

KnaveLayerSurface::KnaveLayerSurface(KnaveLayerShellIntegration *integration,
                                     QtWaylandClient::QWaylandWindow *window)
    : QWaylandShellSurface(window)
{
    const bool overview = qEnvironmentVariable("KNAVE_SHELL_ROLE") == QStringLiteral("overview");
    const uint32_t layer = overview ? QtWayland::zwlr_layer_shell_v1::layer_overlay
                                    : QtWayland::zwlr_layer_shell_v1::layer_top;
    init(integration->get_layer_surface(wlSurface(), nullptr, layer,
                                        overview ? QStringLiteral("knave-overview")
                                                 : QStringLiteral("knave-bar")));

    if (overview) {
        set_size(0, 0);
        set_anchor(anchor_top | anchor_bottom | anchor_left | anchor_right);
        set_exclusive_zone(-1);
        set_keyboard_interactivity(keyboard_interactivity_exclusive);
    } else {
        set_size(0, 36);
        set_anchor(anchor_top | anchor_left | anchor_right);
        set_exclusive_zone(36);
        set_keyboard_interactivity(keyboard_interactivity_none);
    }
    wl_surface_commit(wlSurface());
}

KnaveLayerSurface::~KnaveLayerSurface()
{
    if (isInitialized())
        destroy();
}

void KnaveLayerSurface::zwlr_layer_surface_v1_configure(uint32_t serial,
                                                         uint32_t width,
                                                         uint32_t height)
{
    ack_configure(serial);
    m_configured = true;
    m_pendingSize = QSize(static_cast<int>(width), static_cast<int>(height));
    applyConfigureWhenPossible();
    window()->updateExposure();
}

void KnaveLayerSurface::applyConfigure()
{
    if (m_pendingSize.isValid() && !m_pendingSize.isEmpty())
        resizeFromApplyConfigure(m_pendingSize);
}

void KnaveLayerSurface::zwlr_layer_surface_v1_closed()
{
    QCoreApplication::quit();
}
