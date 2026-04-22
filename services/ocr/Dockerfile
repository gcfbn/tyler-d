FROM python:3.12-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    tesseract-ocr \
    tesseract-ocr-pol \
    libtesseract-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy requirements first for caching
COPY services/ocr/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy root contents (protos) and service code
WORKDIR /usr/src/app
COPY . .

# Move to service dir and generate code
WORKDIR /usr/src/app/services/ocr
RUN python codegen.py

EXPOSE 50051

CMD ["python", "main.py"]
