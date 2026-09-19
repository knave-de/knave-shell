#include "shellservice.h"

#include <QDir>
#include <QFileInfo>
#include <QGuiApplication>
#include <QLocalServer>
#include <QLocalSocket>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QRegularExpression>
#include <QTimer>

namespace {
QString overviewSocketName()
{
    QString display = qEnvironmentVariable("WAYLAND_DISPLAY", "wayland");
    display.replace(QRegularExpression(QStringLiteral("[^A-Za-z0-9_.-]")),
                    QStringLiteral("_"));
    return QStringLiteral("knave-shell-overview-%1").arg(display);
}

bool toggleExistingOverview(const QString &name)
{
    QLocalSocket socket;
    socket.connectToServer(name, QIODevice::WriteOnly);
    if (!socket.waitForConnected(120))
        return false;
    socket.write("toggle\n");
    socket.flush();
    socket.waitForBytesWritten(120);
    return true;
}

void configurePluginPath(const char *argv0)
{
    QFileInfo executable(QStringLiteral("/proc/self/exe"));
    QString executablePath = executable.canonicalFilePath();
    if (executablePath.isEmpty())
        executablePath = QFileInfo(QString::fromLocal8Bit(argv0)).absoluteFilePath();

    const QDir binDirectory(QFileInfo(executablePath).absolutePath());
    QStringList paths{
        binDirectory.absoluteFilePath(QStringLiteral("../plugins")),
        binDirectory.absoluteFilePath(QStringLiteral("../lib/knave-shell/plugins")),
    };
    const QString inherited = qEnvironmentVariable("QT_PLUGIN_PATH");
    if (!inherited.isEmpty())
        paths.append(inherited.split(QDir::listSeparator(), Qt::SkipEmptyParts));
    qputenv("QT_PLUGIN_PATH", paths.join(QDir::listSeparator()).toLocal8Bit());
    qputenv("QT_WAYLAND_SHELL_INTEGRATION", "knave-layer-shell");
    if (!qEnvironmentVariableIsSet("QT_QPA_PLATFORM")
        && qEnvironmentVariableIsSet("WAYLAND_DISPLAY")) {
        qputenv("QT_QPA_PLATFORM", "wayland");
    }
}
}

int main(int argc, char *argv[])
{
    const QString role = argc > 1 ? QString::fromLocal8Bit(argv[1])
                                  : QStringLiteral("bar");
    if (role != QStringLiteral("bar") && role != QStringLiteral("overview")) {
        qCritical("usage: knave-shell [bar|overview]");
        return 2;
    }

    configurePluginPath(argv[0]);
    qputenv("KNAVE_SHELL_ROLE", role.toLocal8Bit());
    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Knave Shell"));
    QGuiApplication::setOrganizationName(QStringLiteral("Knave"));

    QLocalServer overviewServer;
    if (role == QStringLiteral("overview")
        && !qEnvironmentVariableIsSet("KNAVE_SHELL_DISABLE_SINGLE_INSTANCE")) {
        const QString socketName = overviewSocketName();
        if (toggleExistingOverview(socketName))
            return 0;
        QLocalServer::removeServer(socketName);
        if (!overviewServer.listen(socketName)) {
            qCritical("could not create overview control socket: %s",
                      qPrintable(overviewServer.errorString()));
            return 1;
        }
        QObject::connect(&overviewServer, &QLocalServer::newConnection, &app, [&] {
            while (QLocalSocket *socket = overviewServer.nextPendingConnection()) {
                socket->deleteLater();
                QTimer::singleShot(0, &app, &QCoreApplication::quit);
            }
        });
    }

    ShellService service;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("Shell"), &service);
    engine.addImageProvider(QStringLiteral("workspace"),
                            new WorkspaceImageProvider(&service));
    const QUrl source(role == QStringLiteral("overview")
                          ? QStringLiteral("qrc:/qt/qml/KnaveShell/qml/Overview.qml")
                          : QStringLiteral("qrc:/qt/qml/KnaveShell/qml/Bar.qml"));
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
        [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(source);
    return app.exec();
}
