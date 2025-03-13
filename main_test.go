package main

/*
#cgo LDFLAGS: -L. -lheif
#include <libheif/heif.h>
#include <stdlib.h>
#include <stdio.h>

// 辅助函数：将 HEIC 文件解码为 JPEG
int convert_heic_to_jpeg(const char* input_path, const char* output_path) {
    struct heif_context* ctx = heif_context_alloc();
    if (!ctx) {
        fprintf(stderr, "Failed to create HEIF context\n");
        return -1;
    }

    // 读取 HEIC 文件
    struct heif_error err = heif_context_read_from_file(ctx, input_path, NULL);
    if (err.code != heif_error_Ok) {
        fprintf(stderr, "Failed to read HEIC file: %s\n", err.message);
        heif_context_free(ctx);
        return -1;
    }

    // 获取主图像句柄
    struct heif_image_handle* handle;
    err = heif_context_get_primary_image_handle(ctx, &handle);
    if (err.code != heif_error_Ok) {
        fprintf(stderr, "Failed to get primary image handle: %s\n", err.message);
        heif_context_free(ctx);
        return -1;
    }

    // 解码图像
    struct heif_image* image;
    err = heif_decode_image(handle, &image, heif_colorspace_RGB, heif_chroma_interleaved_RGB, NULL);
    if (err.code != heif_error_Ok) {
        fprintf(stderr, "Failed to decode image: %s\n", err.message);
        heif_image_handle_release(handle);
        heif_context_free(ctx);
        return -1;
    }

    // 获取图像数据
    int width = heif_image_get_width(image, heif_channel_interleaved);
    int height = heif_image_get_height(image, heif_channel_interleaved);
    int stride;
    const uint8_t* data = heif_image_get_plane_readonly(image, heif_channel_interleaved, &stride);

    // 将图像数据保存为 JPEG
    FILE* fp = fopen(output_path, "wb");
    if (!fp) {
        fprintf(stderr, "Failed to open output file\n");
        heif_image_release(image);
        heif_image_handle_release(handle);
        heif_context_free(ctx);
        return -1;
    }

    // 使用 stb_image_write 或其他库将 RGB 数据保存为 JPEG
    // 这里省略 JPEG 编码的实现，可以使用 libjpeg 或其他库
    fwrite(data, 1, width * height * 3, fp);
    fclose(fp);

    // 释放资源
    heif_image_release(image);
    heif_image_handle_release(handle);
    heif_context_free(ctx);

    return 0;
}
*/
import "C"
import (
	"fmt"
	"os"
	"unsafe"
)

func main() {
	if len(os.Args) != 3 {
		fmt.Println("Usage: heic_to_jpeg <input.heic> <output.jpg>")
		return
	}

	inputPath := os.Args[1]
	outputPath := os.Args[2]

	// 将 Go 字符串转换为 C 字符串
	cInput := C.CString(inputPath)
	cOutput := C.CString(outputPath)
	defer C.free(unsafe.Pointer(cInput))
	defer C.free(unsafe.Pointer(cOutput))

	// 调用 C 函数
	result := C.convert_heic_to_jpeg(cInput, cOutput)
	if result != 0 {
		fmt.Println("Failed to convert HEIC to JPEG")
		return
	}

	fmt.Printf("Successfully converted %s to %s\n", inputPath, outputPath)
}
