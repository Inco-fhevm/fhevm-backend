// Code generated by protoc-gen-go-grpc. DO NOT EDIT.
// versions:
// - protoc-gen-go-grpc v1.5.1
// - protoc             v5.27.3
// source: coprocessor.proto

package fhevm

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.64.0 or later.
const _ = grpc.SupportPackageIsVersion9

const (
	FhevmCoprocessor_AsyncCompute_FullMethodName              = "/fhevm.coprocessor.FhevmCoprocessor/AsyncCompute"
	FhevmCoprocessor_WaitComputations_FullMethodName          = "/fhevm.coprocessor.FhevmCoprocessor/WaitComputations"
	FhevmCoprocessor_UploadInputs_FullMethodName              = "/fhevm.coprocessor.FhevmCoprocessor/UploadInputs"
	FhevmCoprocessor_GetCiphertexts_FullMethodName            = "/fhevm.coprocessor.FhevmCoprocessor/GetCiphertexts"
	FhevmCoprocessor_TrivialEncryptCiphertexts_FullMethodName = "/fhevm.coprocessor.FhevmCoprocessor/TrivialEncryptCiphertexts"
)

// FhevmCoprocessorClient is the client API for FhevmCoprocessor service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type FhevmCoprocessorClient interface {
	AsyncCompute(ctx context.Context, in *AsyncComputeRequest, opts ...grpc.CallOption) (*GenericResponse, error)
	WaitComputations(ctx context.Context, in *AsyncComputeRequest, opts ...grpc.CallOption) (*FhevmResponses, error)
	UploadInputs(ctx context.Context, in *InputUploadBatch, opts ...grpc.CallOption) (*InputUploadResponse, error)
	GetCiphertexts(ctx context.Context, in *GetCiphertextBatch, opts ...grpc.CallOption) (*GetCiphertextResponse, error)
	TrivialEncryptCiphertexts(ctx context.Context, in *TrivialEncryptBatch, opts ...grpc.CallOption) (*GenericResponse, error)
}

type fhevmCoprocessorClient struct {
	cc grpc.ClientConnInterface
}

func NewFhevmCoprocessorClient(cc grpc.ClientConnInterface) FhevmCoprocessorClient {
	return &fhevmCoprocessorClient{cc}
}

func (c *fhevmCoprocessorClient) AsyncCompute(ctx context.Context, in *AsyncComputeRequest, opts ...grpc.CallOption) (*GenericResponse, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(GenericResponse)
	err := c.cc.Invoke(ctx, FhevmCoprocessor_AsyncCompute_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *fhevmCoprocessorClient) WaitComputations(ctx context.Context, in *AsyncComputeRequest, opts ...grpc.CallOption) (*FhevmResponses, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(FhevmResponses)
	err := c.cc.Invoke(ctx, FhevmCoprocessor_WaitComputations_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *fhevmCoprocessorClient) UploadInputs(ctx context.Context, in *InputUploadBatch, opts ...grpc.CallOption) (*InputUploadResponse, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(InputUploadResponse)
	err := c.cc.Invoke(ctx, FhevmCoprocessor_UploadInputs_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *fhevmCoprocessorClient) GetCiphertexts(ctx context.Context, in *GetCiphertextBatch, opts ...grpc.CallOption) (*GetCiphertextResponse, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(GetCiphertextResponse)
	err := c.cc.Invoke(ctx, FhevmCoprocessor_GetCiphertexts_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *fhevmCoprocessorClient) TrivialEncryptCiphertexts(ctx context.Context, in *TrivialEncryptBatch, opts ...grpc.CallOption) (*GenericResponse, error) {
	cOpts := append([]grpc.CallOption{grpc.StaticMethod()}, opts...)
	out := new(GenericResponse)
	err := c.cc.Invoke(ctx, FhevmCoprocessor_TrivialEncryptCiphertexts_FullMethodName, in, out, cOpts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// FhevmCoprocessorServer is the server API for FhevmCoprocessor service.
// All implementations must embed UnimplementedFhevmCoprocessorServer
// for forward compatibility.
type FhevmCoprocessorServer interface {
	AsyncCompute(context.Context, *AsyncComputeRequest) (*GenericResponse, error)
	WaitComputations(context.Context, *AsyncComputeRequest) (*FhevmResponses, error)
	UploadInputs(context.Context, *InputUploadBatch) (*InputUploadResponse, error)
	GetCiphertexts(context.Context, *GetCiphertextBatch) (*GetCiphertextResponse, error)
	TrivialEncryptCiphertexts(context.Context, *TrivialEncryptBatch) (*GenericResponse, error)
	mustEmbedUnimplementedFhevmCoprocessorServer()
}

// UnimplementedFhevmCoprocessorServer must be embedded to have
// forward compatible implementations.
//
// NOTE: this should be embedded by value instead of pointer to avoid a nil
// pointer dereference when methods are called.
type UnimplementedFhevmCoprocessorServer struct{}

func (UnimplementedFhevmCoprocessorServer) AsyncCompute(context.Context, *AsyncComputeRequest) (*GenericResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method AsyncCompute not implemented")
}
func (UnimplementedFhevmCoprocessorServer) WaitComputations(context.Context, *AsyncComputeRequest) (*FhevmResponses, error) {
	return nil, status.Errorf(codes.Unimplemented, "method WaitComputations not implemented")
}
func (UnimplementedFhevmCoprocessorServer) UploadInputs(context.Context, *InputUploadBatch) (*InputUploadResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method UploadInputs not implemented")
}
func (UnimplementedFhevmCoprocessorServer) GetCiphertexts(context.Context, *GetCiphertextBatch) (*GetCiphertextResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GetCiphertexts not implemented")
}
func (UnimplementedFhevmCoprocessorServer) TrivialEncryptCiphertexts(context.Context, *TrivialEncryptBatch) (*GenericResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method TrivialEncryptCiphertexts not implemented")
}
func (UnimplementedFhevmCoprocessorServer) mustEmbedUnimplementedFhevmCoprocessorServer() {}
func (UnimplementedFhevmCoprocessorServer) testEmbeddedByValue()                          {}

// UnsafeFhevmCoprocessorServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to FhevmCoprocessorServer will
// result in compilation errors.
type UnsafeFhevmCoprocessorServer interface {
	mustEmbedUnimplementedFhevmCoprocessorServer()
}

func RegisterFhevmCoprocessorServer(s grpc.ServiceRegistrar, srv FhevmCoprocessorServer) {
	// If the following call pancis, it indicates UnimplementedFhevmCoprocessorServer was
	// embedded by pointer and is nil.  This will cause panics if an
	// unimplemented method is ever invoked, so we test this at initialization
	// time to prevent it from happening at runtime later due to I/O.
	if t, ok := srv.(interface{ testEmbeddedByValue() }); ok {
		t.testEmbeddedByValue()
	}
	s.RegisterService(&FhevmCoprocessor_ServiceDesc, srv)
}

func _FhevmCoprocessor_AsyncCompute_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(AsyncComputeRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(FhevmCoprocessorServer).AsyncCompute(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: FhevmCoprocessor_AsyncCompute_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(FhevmCoprocessorServer).AsyncCompute(ctx, req.(*AsyncComputeRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _FhevmCoprocessor_WaitComputations_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(AsyncComputeRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(FhevmCoprocessorServer).WaitComputations(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: FhevmCoprocessor_WaitComputations_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(FhevmCoprocessorServer).WaitComputations(ctx, req.(*AsyncComputeRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _FhevmCoprocessor_UploadInputs_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(InputUploadBatch)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(FhevmCoprocessorServer).UploadInputs(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: FhevmCoprocessor_UploadInputs_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(FhevmCoprocessorServer).UploadInputs(ctx, req.(*InputUploadBatch))
	}
	return interceptor(ctx, in, info, handler)
}

func _FhevmCoprocessor_GetCiphertexts_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(GetCiphertextBatch)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(FhevmCoprocessorServer).GetCiphertexts(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: FhevmCoprocessor_GetCiphertexts_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(FhevmCoprocessorServer).GetCiphertexts(ctx, req.(*GetCiphertextBatch))
	}
	return interceptor(ctx, in, info, handler)
}

func _FhevmCoprocessor_TrivialEncryptCiphertexts_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(TrivialEncryptBatch)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(FhevmCoprocessorServer).TrivialEncryptCiphertexts(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: FhevmCoprocessor_TrivialEncryptCiphertexts_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(FhevmCoprocessorServer).TrivialEncryptCiphertexts(ctx, req.(*TrivialEncryptBatch))
	}
	return interceptor(ctx, in, info, handler)
}

// FhevmCoprocessor_ServiceDesc is the grpc.ServiceDesc for FhevmCoprocessor service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var FhevmCoprocessor_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "fhevm.coprocessor.FhevmCoprocessor",
	HandlerType: (*FhevmCoprocessorServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "AsyncCompute",
			Handler:    _FhevmCoprocessor_AsyncCompute_Handler,
		},
		{
			MethodName: "WaitComputations",
			Handler:    _FhevmCoprocessor_WaitComputations_Handler,
		},
		{
			MethodName: "UploadInputs",
			Handler:    _FhevmCoprocessor_UploadInputs_Handler,
		},
		{
			MethodName: "GetCiphertexts",
			Handler:    _FhevmCoprocessor_GetCiphertexts_Handler,
		},
		{
			MethodName: "TrivialEncryptCiphertexts",
			Handler:    _FhevmCoprocessor_TrivialEncryptCiphertexts_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "coprocessor.proto",
}
