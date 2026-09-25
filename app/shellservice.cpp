#include "shellservice.h"

#include <QColor>
#include <QCoreApplication>
#include <QProcess>
#include <QVariantMap>

namespace {
QString textOrFallback(const char *value, const QString &fallback)
{
    const QString text = QString::fromUtf8(value);
    return text.isEmpty() ? fallback : text;
}
}

ShellService::ShellService(QObject *parent)
    : QObject(parent), m_core(ks_core_new())
{
    m_refreshTimer.setInterval(750);
    connect(&m_refreshTimer, &QTimer::timeout, this, &ShellService::refresh);
    m_refreshTimer.start();
    QTimer::singleShot(0, this, &ShellService::refresh);
}

ShellService::~ShellService()
{
    ks_core_free(m_core);
}

void ShellService::refresh()
{
    ks_core_refresh(m_core);
    m_connected = ks_core_connected(m_core);
    m_lastError = QString::fromUtf8(ks_core_last_error(m_core));
    m_previewRevision = ks_core_preview_revision(m_core);

    QVariantList workspaces;
    const size_t workspaceCount = ks_core_workspace_count(m_core);
    for (size_t index = 0; index < workspaceCount; ++index) {
        QVariantMap workspace;
        const uint number = ks_core_workspace_number(m_core, index);
        const bool active = ks_core_workspace_active(m_core, index);
        workspace.insert(QStringLiteral("number"), number);
        workspace.insert(QStringLiteral("active"), active);
        workspace.insert(QStringLiteral("windowCount"),
                         qulonglong(ks_core_workspace_window_count(m_core, index)));
        workspace.insert(QStringLiteral("visibleWindowCount"),
                         qulonglong(ks_core_workspace_visible_window_count(m_core, index)));
        workspaces.append(workspace);
        if (active)
            m_activeWorkspace = static_cast<int>(number);
    }
    if (workspaces.isEmpty()) {
        for (uint number = 1; number <= 10; ++number) {
            workspaces.append(QVariantMap{
                {QStringLiteral("number"), number},
                {QStringLiteral("active"), number == 1},
                {QStringLiteral("windowCount"), 0},
                {QStringLiteral("visibleWindowCount"), 0},
            });
        }
        m_activeWorkspace = 1;
    }
    m_workspaces = workspaces;

    QVariantList windows;
    const size_t windowCount = ks_core_window_count(m_core);
    for (size_t index = 0; index < windowCount; ++index) {
        const QString appId = QString::fromUtf8(ks_core_window_app_id(m_core, index));
        QVariantMap window;
        window.insert(QStringLiteral("id"),
                      QString::number(ks_core_window_id(m_core, index)));
        window.insert(QStringLiteral("title"),
                      textOrFallback(ks_core_window_title(m_core, index),
                                     appId.isEmpty() ? tr("Untitled window") : appId));
        window.insert(QStringLiteral("appId"), appId);
        window.insert(QStringLiteral("workspace"),
                      ks_core_window_workspace(m_core, index));
        window.insert(QStringLiteral("minimized"),
                      ks_core_window_minimized(m_core, index));
        window.insert(QStringLiteral("floating"),
                      ks_core_window_floating(m_core, index));
        window.insert(QStringLiteral("fullscreen"),
                      ks_core_window_fullscreen(m_core, index));
        window.insert(QStringLiteral("focused"),
                      ks_core_window_focused(m_core, index));
        windows.append(window);
    }
    m_windows = windows;
    emit stateChanged();
}

void ShellService::focusWorkspace(uint workspace)
{
    if (ks_core_focus_workspace(m_core, workspace))
        refresh();
    else {
        m_connected = ks_core_connected(m_core);
        m_lastError = QString::fromUtf8(ks_core_last_error(m_core));
        emit stateChanged();
    }
}

void ShellService::focusWindow(const QString &windowId)
{
    bool valid = false;
    const qulonglong id = windowId.toULongLong(&valid);
    if (valid && ks_core_focus_window(m_core, id))
        refresh();
    else {
        m_connected = ks_core_connected(m_core);
        m_lastError = valid ? QString::fromUtf8(ks_core_last_error(m_core))
                            : tr("Invalid window identifier");
        emit stateChanged();
    }
}

void ShellService::openOverview()
{
    QProcess::startDetached(QCoreApplication::applicationFilePath(),
                            {QStringLiteral("overview")});
}

void ShellService::closeOverview()
{
    QCoreApplication::quit();
}

QImage ShellService::preview(uint workspace) const
{
    size_t length = 0;
    const uint8_t *bytes = ks_core_preview_png(m_core, workspace, &length);
    if (!bytes || length == 0)
        return {};
    return QImage::fromData(bytes, qsizetype(length), "PNG");
}

WorkspaceImageProvider::WorkspaceImageProvider(const ShellService *service)
    : QQuickImageProvider(QQuickImageProvider::Image), m_service(service)
{
}

QImage WorkspaceImageProvider::requestImage(const QString &id, QSize *size,
                                             const QSize &requestedSize)
{
    bool valid = false;
    const uint workspace = id.section(QLatin1Char('/'), 0, 0).toUInt(&valid);
    QImage image = valid ? m_service->preview(workspace) : QImage();
    if (image.isNull()) {
        image = QImage(QSize(480, 270), QImage::Format_RGBA8888);
        image.fill(QColor(QStringLiteral("#151d28")));
    }
    if (size)
        *size = image.size();
    if (requestedSize.isValid())
        return image.scaled(requestedSize, Qt::KeepAspectRatio,
                            Qt::SmoothTransformation);
    return image;
}
