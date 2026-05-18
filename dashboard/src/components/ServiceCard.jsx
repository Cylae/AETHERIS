import React from 'react';
import { ExternalLink } from 'lucide-react';

const ServiceCard = ({ title, description, url, port, icon: Icon }) => {
  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      className="block p-6 bg-white dark:bg-gray-800 rounded-xl shadow-sm hover:shadow-md transition-all duration-200 border border-gray-100 dark:border-gray-700 group"
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center space-x-4">
          <div className="p-3 bg-aetheris-50 dark:bg-aetheris-900/30 rounded-lg text-aetheris-500 dark:text-aetheris-400">
            {Icon && <Icon size={24} />}
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white group-hover:text-aetheris-500 dark:group-hover:text-aetheris-400 transition-colors">
              {title}
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              {description}
            </p>
          </div>
        </div>
        <div className="flex items-center space-x-2 text-gray-400 group-hover:text-aetheris-500 dark:group-hover:text-aetheris-400 transition-colors">
          <span className="text-xs font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">
            Port {port}
          </span>
          <ExternalLink size={18} />
        </div>
      </div>
    </a>
  );
};

export default ServiceCard;
