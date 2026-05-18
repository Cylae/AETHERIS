import React, { useState, useEffect } from 'react';
import { Settings, Server, Mail, HardDrive, Shield, Activity, GitBranch, Link, Sun, Moon } from 'lucide-react';
import ServiceCard from './components/ServiceCard';

function App() {
  const [darkMode, setDarkMode] = useState(false);

  useEffect(() => {
    // Check local storage or system preference on load
    if (localStorage.theme === 'dark' || (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      setDarkMode(true);
      document.documentElement.classList.add('dark');
    } else {
      setDarkMode(false);
      document.documentElement.classList.remove('dark');
    }
  }, []);

  const toggleDarkMode = () => {
    if (darkMode) {
      document.documentElement.classList.remove('dark');
      localStorage.theme = 'light';
      setDarkMode(false);
    } else {
      document.documentElement.classList.add('dark');
      localStorage.theme = 'dark';
      setDarkMode(true);
    }
  };

  const services = [
    {
      title: "Nginx Proxy Manager",
      description: "Manage reverse proxy hosts and SSL certificates centrally.",
      url: "http://localhost:81",
      port: "81",
      icon: Shield
    },
    {
      title: "Portainer",
      description: "Docker environment orchestration and container management.",
      url: "http://localhost:9000",
      port: "9000",
      icon: Server
    },
    {
      title: "Grafana",
      description: "Real-time metrics, analytics, and monitoring dashboards.",
      url: "http://localhost:3000",
      port: "3000",
      icon: Activity
    },
    {
      title: "Nextcloud",
      description: "Private file sync, share, and collaboration platform.",
      url: "http://localhost:8080", // Assuming Nextcloud is mapped to 8080 if not proxied directly, adjust as needed or document proxy requirement
      port: "80 (Container)",
      icon: HardDrive
    },
    {
      title: "Roundcube",
      description: "Webmail interface for the AETHERIS mail server.",
      url: "http://localhost:8000", // Adjust as needed
      port: "80 (Container)",
      icon: Mail
    },
    {
      title: "Gitea",
      description: "Painless self-hosted Git service.",
      url: "http://localhost:3000", // Gitea is often 3000, conflicts with Grafana in bare setup unless proxied
      port: "3000 (Container)",
      icon: GitBranch
    },
    {
      title: "Vaultwarden",
      description: "Unofficial Bitwarden compatible server.",
      url: "http://localhost:80", // Needs proxy
      port: "80 (Container)",
      icon: Shield
    },
    {
      title: "Yourls",
      description: "Your Own URL Shortener.",
      url: "http://localhost:80", // Needs proxy
      port: "80 (Container)",
      icon: Link
    }
  ];

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 bg-aetheris-600 dark:bg-aetheris-500 rounded-lg flex items-center justify-center shadow-inner">
              <Server className="text-white" size={24} />
            </div>
            <div>
              <h1 className="text-xl font-bold text-gray-900 dark:text-white tracking-tight">AETHERIS</h1>
              <p className="text-xs text-gray-500 dark:text-gray-400">Environment-Agnostic Orchestrator</p>
            </div>
          </div>
          <button
            onClick={toggleDarkMode}
            className="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            aria-label="Toggle Dark Mode"
          >
            {darkMode ? <Sun size={20} /> : <Moon size={20} />}
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="mb-8">
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">System Dashboard</h2>
          <p className="text-gray-600 dark:text-gray-400">Access and monitor your containerized infrastructure.</p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {services.map((service, index) => (
            <ServiceCard key={index} {...service} />
          ))}
        </div>
      </main>
    </div>
  );
}

export default App;
