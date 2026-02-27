// Service Worker for Velamen push notifications

// Activate immediately — don't wait for existing tabs to close
self.addEventListener('install', function(event) {
    console.log('[SW] Installing');
    self.skipWaiting();
});

// Take control of all clients immediately
self.addEventListener('activate', function(event) {
    console.log('[SW] Activating');
    event.waitUntil(self.clients.claim());
});

self.addEventListener('push', function(event) {
    console.log('[SW] Push received:', event.data ? event.data.text() : 'no data');

    let data = { title: 'Velamen', body: 'You have a new alert', url: '/' };

    if (event.data) {
        try {
            data = event.data.json();
        } catch (e) {
            data.body = event.data.text();
        }
    }

    const options = {
        body: data.body,
        icon: '/pkg/favicon.ico',
        badge: '/pkg/favicon.ico',
        data: { url: data.url || '/' },
        vibrate: [100, 50, 100],
    };

    event.waitUntil(
        self.registration.showNotification(data.title || 'Velamen', options)
    );
});

self.addEventListener('notificationclick', function(event) {
    event.notification.close();

    const url = event.notification.data && event.notification.data.url
        ? event.notification.data.url
        : '/';

    event.waitUntil(
        clients.matchAll({ type: 'window', includeUncontrolled: true }).then(function(clientList) {
            for (const client of clientList) {
                if (client.url.includes(url) && 'focus' in client) {
                    return client.focus();
                }
            }
            if (clients.openWindow) {
                return clients.openWindow(url);
            }
        })
    );
});
