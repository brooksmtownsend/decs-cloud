import React, { Component } from 'react';
import {    
    Card,
    CardBody,    
    CardHeader,    
    Col,        
    Row    
  } from 'reactstrap';
import Shard from './Shard';

class ShardList extends Component {
    onUpdate = () => {
		this.setState({});
	}

	componentDidMount() {
		this.props.shards.on('add', this.onUpdate);
		this.props.shards.on('remove', this.onUpdate);
	}

	componentWillUnmount() {
		this.props.shards.off('add', this.onUpdate);
		this.props.shards.off('remove', this.onUpdate);
    }
    
    render() {
		// ResClient Collections are iterables, but not arrays.
		return (
        <Row>
          <Col>
            <Card>
              <CardHeader>
                Shards
              </CardHeader>
              <CardBody>
              <Row>
                  <Col>
                  <hr className="mt-0" />
                   
                    <ul>
                      {this.props.shards && Array.from(this.props.shards).map( shard =>
                        <Shard key={shard.name} shards={this.props.shards} shard={shard} />
                      )}                      
                    </ul>
                  
                    <hr className="mt-0" />
                  </Col>
                </Row>                
              </CardBody>
            </Card>
          </Col>
        </Row>			
    );
  }
}

export default ShardList;