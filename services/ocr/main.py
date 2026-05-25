import grpc
from concurrent import futures
import ocr_pb2
import ocr_pb2_grpc
import pytesseract
from PIL import Image
import io
import fitz  # PyMuPDF

class OcrService(ocr_pb2_grpc.OcrServiceServicer):
    def ProcessDocument(self, request, context):
        print(f"Received document of type: {request.file_type}")
        extracted_text = ""
        metadata = {}

        try:
            if request.file_type.lower() == "pdf":
                extracted_text, metadata = self._process_pdf(request.content)
            elif request.file_type.lower() in ["png", "jpg", "jpeg"]:
                extracted_text = self._process_image(request.content)
            else:
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details(f"Unsupported file type: {request.file_type}")
                return ocr_pb2.OcrResponse()

            return ocr_pb2.OcrResponse(text=extracted_text, metadata=metadata)
        except Exception as e:
            print(f"Error processing document: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return ocr_pb2.OcrResponse()

    def _process_image(self, content):
        image = Image.open(io.BytesIO(content))
        
        # Pre-processing for better OCR: Convert to Grayscale
        image = image.convert('L') 
        
        # Use Polish and English for OCR with slightly more configuration
        # --psm 3: Fully automatic page segmentation, but no OSD. (Default)
        text = pytesseract.image_to_string(image, lang='pol+eng', config='--psm 3')
        return text.strip()

    def _process_pdf(self, content):
        doc = fitz.open(stream=content, filetype="pdf")
        text_parts = []
        
        # Extract native text if available
        for page in doc:
            text_parts.append(page.get_text())
        
        full_text = "\n".join(text_parts).strip()
        
        # If no text found, try OCR on the first page as a fallback
        if not full_text and len(doc) > 0:
            print("No native text found in PDF, falling back to OCR on first page...")
            page = doc[0]
            pix = page.get_pixmap()
            img = Image.open(io.BytesIO(pix.tobytes()))
            full_text = pytesseract.image_to_string(img, lang='pol+eng')

        metadata = {
            "page_count": str(len(doc)),
            "title": doc.metadata.get("title", ""),
            "author": doc.metadata.get("author", ""),
        }
        
        return full_text, metadata

import os

def serve():
    port = os.environ.get("OCR_PORT", "50051")
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    ocr_pb2_grpc.add_OcrServiceServicer_to_server(OcrService(), server)
    server.add_insecure_port(f'[::]:{port}')
    print(f"OCR Service starting on port {port}...")
    server.start()
    server.wait_for_termination()

if __name__ == "__main__":
    serve()
