#ifndef KNAVE_LAYER_SHELL_INTEGRATION_H
#define KNAVE_LAYER_SHELL_INTEGRATION_H

#include <QSize>
#include <QtWaylandClient/private/qwaylandclientshellapi_p.h>

#include "qwayland-wlr-layer-shell-unstable-v1.h"

class KnaveLayerShellIntegration final
    : public QtWaylandClient::QWaylandShellIntegrationTemplate<KnaveLayerShellIntegration>,
      public QtWayland::zwlr_layer_shell_v1
{
public:
    KnaveLayerShellIntegration();

    QtWaylandClient::QWaylandShellSurface *createShellSurface(
        QtWaylandClient::QWaylandWindow *window) override;
};

class KnaveLayerSurface final
    : public QtWaylandClient::QWaylandShellSurface,
      public QtWayland::zwlr_layer_surface_v1
{
public:
    KnaveLayerSurface(KnaveLayerShellIntegration *integration,
                      QtWaylandClient::QWaylandWindow *window);
    ~KnaveLayerSurface() override;

    bool isExposed() const override { return m_configured; }
    bool wantsDecorations() const override { return false; }
    void applyConfigure() override;

protected:
    void zwlr_layer_surface_v1_configure(uint32_t serial,
                                         uint32_t width,
                                         uint32_t height) override;
    void zwlr_layer_surface_v1_closed() override;

private:
    bool m_configured = false;
    QSize m_pendingSize;
};

#endif
