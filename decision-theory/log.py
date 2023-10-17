# Local modules
import pretty

class Logger:
    def __init__(self):
        self.indent_level = 0

    def group(self, message):
        indent = pretty.indent(self.indent_level)
        print(f"{indent}{message}")

        logger = self
        class ContextManager:
            def __enter__(self):
                logger.indent_level += 1
            def __exit__(self, _1, _2, _3):
                logger.indent_level -= 1
                if logger.indent_level < 0:
                    raise Exception("Logger: too many 'close()'s")

        return ContextManager()
    
    def log(self, message):
        indent = pretty.indent(self.indent_level)
        print(f"{indent}{message}")
