#ifndef KNAVE_SHELL_SERVICE_H
#define KNAVE_SHELL_SERVICE_H

#include <QImage>
#include <QObject>
#include <QQuickImageProvider>
#include <QTimer>
#include <QVariantList>

#include "knave_shell_core.h"

class ShellService final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QVariantList workspaces READ workspaces NOTIFY stateChanged)
    Q_PROPERTY(QVariantList windows READ windows NOTIFY stateChanged)
    Q_PROPERTY(int activeWorkspace READ activeWorkspace NOTIFY stateChanged)
    Q_PROPERTY(bool connected READ connected NOTIFY stateChanged)
    Q_PROPERTY(QString lastError READ lastError NOTIFY stateChanged)
    Q_PROPERTY(qulonglong previewRevision READ previewRevision NOTIFY stateChanged)

public:
    explicit ShellService(QObject *parent = nullptr);
    ~ShellService() override;

    QVariantList workspaces() const { return m_workspaces; }
    QVariantList windows() const { return m_windows; }
    int activeWorkspace() const { return m_activeWorkspace; }
    bool connected() const { return m_connected; }
    QString lastError() const { return m_lastError; }
    qulonglong previewRevision() const { return m_previewRevision; }

    QImage preview(uint workspace) const;

    Q_INVOKABLE void refresh();
    Q_INVOKABLE void focusWorkspace(uint workspace);
    Q_INVOKABLE void focusWindow(const QString &windowId);
    Q_INVOKABLE void openOverview();
    Q_INVOKABLE void closeOverview();

signals:
    void stateChanged();

private:
    KsCore *m_core = nullptr;
    QTimer m_refreshTimer;
    QVariantList m_workspaces;
    QVariantList m_windows;
    int m_activeWorkspace = 1;
    bool m_connected = false;
    QString m_lastError;
    qulonglong m_previewRevision = 0;
};

class WorkspaceImageProvider final : public QQuickImageProvider
{
public:
    explicit WorkspaceImageProvider(const ShellService *service);

    QImage requestImage(const QString &id, QSize *size,
                        const QSize &requestedSize) override;

private:
    const ShellService *m_service;
};

#endif
