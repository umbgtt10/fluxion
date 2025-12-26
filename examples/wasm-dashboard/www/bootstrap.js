// Bootstrap the WASM module
import init, { start_dashboard } from './pkg/wasm_dashboard.js';

async function run() {
    try {
        console.log('Loading WASM module...');

        // Initialize the WASM module
        await init();
        console.log('WASM module loaded successfully');

        // Start the dashboard
        console.log('Starting dashboard...');
        await start_dashboard();
        console.log('Dashboard started successfully');

    } catch (error) {
        console.error('Failed to initialize dashboard:', error);

        // Show error to user
        const errorDiv = document.createElement('div');
        errorDiv.style.cssText = `
            position: fixed;
            top: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: #e74c3c;
            color: white;
            padding: 1rem 2rem;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
            z-index: 1000;
            max-width: 90%;
        `;
        errorDiv.textContent = `Failed to load dashboard: ${error.message}`;
        document.body.appendChild(errorDiv);
    }
}

// Run when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', run);
} else {
    run();
}
