(function() {var implementors = {};
implementors["bytes"] = [];
implementors["hyper"] = [];
implementors["libc"] = [];
implementors["mio"] = [];
implementors["rand"] = [];
implementors["regex_syntax"] = [];
implementors["thread_local"] = [];
implementors["tokio_core"] = [];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
