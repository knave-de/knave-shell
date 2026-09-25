#include "layershellintegration.h"

#include <QtWaylandClient/private/qwaylandclientshellapi_p.h>

class KnaveLayerShellPlugin final
    : public QtWaylandClient::QWaylandShellIntegrationPlugin
{
    Q_OBJECT
    Q_PLUGIN_METADATA(IID QWaylandShellIntegrationFactoryInterface_iid
                      FILE "knave-layer-shell.json")

public:
    QtWaylandClient::QWaylandShellIntegration *create(
        const QString &key, const QStringList &parameters) override
    {
        Q_UNUSED(key)
        Q_UNUSED(parameters)
        return new KnaveLayerShellIntegration;
    }
};

#include "plugin.moc"
