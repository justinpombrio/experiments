class Logger:
    def __init__(self):
        self.indentation = []
    
    def log(self, message):
        print(f"{''.join(self.indentation)}{message}")

    def group(self, message, marker=" "):
        print(f"{''.join(self.indentation)}{message}")

        logger = self
        class ContextManager:
            def __enter__(self):
                logger.indentation.append(f"{marker}   ")
            def __exit__(self, _1, _2, _3):
                logger.indentation.pop()

        return ContextManager()
