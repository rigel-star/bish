class StrSplit:
    def __init__(self, data: str, delimiter: str):
        self.remainder = data
        self.delimiter = delimiter

    def __iter__(self):
        return self
    
    def __next__(self):
        if self.delimiter == "":
            raise StopIteration
        if self.remainder.find(self.delimiter) == -1:
            return self.delimiter
        self.delimiter.split(self.delimiter)


if __name__ == "__main__":
    for s in StrSplit("a b c d e", " "):
        print(s)